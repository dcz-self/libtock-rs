mod elf2tab;
mod output_processor;
mod qemu;
mod tockloader;

use clap::{ArgEnum, Parser};
use std::env::{var, VarError};
use std::path::PathBuf;

/// Converts ELF binaries into Tock Binary Format binaries and runs them on a
/// Tock system.
#[derive(Debug, Parser)]
pub struct Cli {
    /// Where to deploy the process binary. If not specified, runner will only
    /// make a TBF file and not attempt to run it.
    #[clap(arg_enum, long, short)]
    deploy: Option<Deploy>,

    /// The executable to convert into Tock Binary Format and run.
    elf: PathBuf,

    /// Whether to output verbose debugging information to the console.
    #[clap(long, short)]
    verbose: bool,
}

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum Deploy {
    Qemu,
    Tockloader,
}

fn main() {
    let cli = Cli::parse();

    let arch = match var("LIBTOCK_ARCH") {
        Err(VarError::NotPresent) => {
            panic!("LIBTOCK_ARCH must be specified to deploy")
        }
        Err(VarError::NotUnicode(arch)) => {
            panic!("Non-UTF-8 LIBTOCK_ARCH value: {:?}", arch)
        }
        Ok(arch) => arch,
    };
    if cli.verbose {
        println!("Detected arch {}", arch);
    }

    let paths = elf2tab::convert_elf(&cli, &arch);
    let deploy = match cli.deploy {
        None => return,
        Some(deploy) => deploy,
    };
    let platform = match var("LIBTOCK_PLATFORM") {
        Err(VarError::NotPresent) => {
            panic!("LIBTOCK_PLATFORM must be specified to deploy")
        }
        Err(VarError::NotUnicode(platform)) => {
            panic!("Non-UTF-8 LIBTOCK_PLATFORM value: {:?}", platform)
        }
        Ok(platform) => platform,
    };
    if cli.verbose {
        println!("Detected platform {}", platform);
    }
    let child = match deploy {
        Deploy::Qemu => qemu::deploy(&cli, platform, paths.tbf_path),
        Deploy::Tockloader => tockloader::deploy(&cli, platform, paths.tab_path),
    };
    output_processor::process(&cli, child);
}
