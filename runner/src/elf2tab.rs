use super::Cli;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::{metadata, remove_file};
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;

// Converts the ELF file specified on the command line into TBF and TAB files,
// and returns the paths to those files.
pub fn convert_elf(cli: &Cli, arch: &str) -> OutFiles {
    let package_name = cli.elf.file_stem().expect("ELF must be a file");
    let mut tab_path = cli.elf.clone();
    tab_path.set_extension("tab");
    let protected_size = TBF_HEADER_SIZE.to_string();
    if cli.verbose {
        println!("Package name: {:?}", package_name);
        println!("TAB path: {}", tab_path.display());
        println!("Protected region size: {}", protected_size);
    }
    let stack_size = read_stack_size(cli);
    let elf_path: PathBuf = cli.elf.as_os_str().into();
    // Tockloader expects the .tbf files inside the .tab
    // to start with an architecture name.
    // elf2tab doesn't give an option to choose them,
    // but relies on the elf name,
    // so here a correctly named elf is supplied directly.
    let mut elf = cli.elf.clone();
    let mut elf_name = OsString::from(arch);
    elf_name.push(".");
    elf_name.push(elf_path.file_name().unwrap_or_else(|| OsStr::new("")));
    elf_name.push(".elf");
    elf.set_file_name(elf_name);

    let mut tbf_path = elf.clone();
    tbf_path.set_extension("tbf");
    if cli.verbose {
        println!("ELF file: {:?}", elf);
        println!("TBF path: {}", tbf_path.display());
    }

    fs::copy(&elf_path, &elf).expect("Couldn't copy the elf file");

    // If elf2tab returns a successful status but does not write to the TBF
    // file, then we run the risk of using an outdated TBF file, creating a
    // hard-to-debug situation. Therefore, we delete the TBF file, forcing
    // elf2tab to create it, and later verify that it exists.
    if let Err(io_error) = remove_file(&tbf_path) {
        // Ignore file-no-found errors, panic on any other error.
        if io_error.kind() != ErrorKind::NotFound {
            panic!("Unable to remove the TBF file. Error: {}", io_error);
        }
    }

    let mut command = Command::new("elf2tab");
    #[rustfmt::skip]
    command.args([
        // TODO: libtock-rs' crates are designed for Tock 2.1's Allow interface,
        // so we should increment this as soon as the Tock kernel will accept a
        // 2.1 app.
        "--kernel-major".as_ref(), "2".as_ref(),
        "--kernel-minor".as_ref(), "0".as_ref(),
        "-n".as_ref(), package_name,
        "-o".as_ref(), tab_path.as_os_str(),
        "--protected-region-size".as_ref(), protected_size.as_ref(),
        "--stack".as_ref(), stack_size.as_ref(),
        elf.as_os_str(),
    ]);
    if cli.verbose {
        command.arg("-v");
        println!("elf2tab command: {:?}", command);
        println!("Spawning elf2tab");
    }
    let mut child = command.spawn().expect("failed to spawn elf2tab");
    let status = child.wait().expect("failed to wait for elf2tab");
    if cli.verbose {
        println!("elf2tab finished. {}", status);
    }
    assert!(status.success(), "elf2tab returned an error. {}", status);

    // Verify that elf2tab created the TBF file, and that it is a file.
    match metadata(&tbf_path) {
        Err(io_error) => {
            if io_error.kind() == ErrorKind::NotFound {
                panic!("elf2tab did not create {}", tbf_path.display());
            }
            panic!(
                "Unable to query metadata for {}: {}",
                tbf_path.display(),
                io_error
            );
        }
        Ok(metadata) => {
            assert!(metadata.is_file(), "{} is not a file", tbf_path.display());
        }
    }

    OutFiles { tab_path, tbf_path }
}

// Paths to the files output by elf2tab.
pub struct OutFiles {
    pub tab_path: PathBuf,
    pub tbf_path: PathBuf,
}

// The amount of space to reserve for the TBF header. This must match the
// TBF_HEADER_SIZE value in the layout file for the platform, which is currently
// 0x48 for all platforms.
const TBF_HEADER_SIZE: u32 = 0x48;

// Reads the stack size, and returns it as a String for use on elf2tab's command
// line.
fn read_stack_size(cli: &Cli) -> String {
    let file = elf::File::open_path(&cli.elf).expect("Unable to open ELF");
    for section in file.sections {
        // This section name comes from runtime/libtock_layout.ld, and it
        // matches the size (and location) of the process binary's stack.
        if section.shdr.name == ".stack" {
            let stack_size = section.shdr.size.to_string();
            if cli.verbose {
                println!("Found .stack section, size: {}", stack_size);
            }
            return stack_size;
        }
    }

    panic!("Unable to find the .stack section in {}", cli.elf.display());
}
