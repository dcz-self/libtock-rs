//! Runtime components related to process startup.

use crate::TockSyscalls;
use libtock_platform::Termination;

// Include the correct `start` symbol (the program entry point) for the
// architecture.
#[cfg(target_arch = "arm")]
core::arch::global_asm!(include_str!("asm_arm.s"));
#[cfg(target_arch = "riscv32")]
core::arch::global_asm!(include_str!("asm_riscv32.s"));

/// `set_main!` is used to tell `libtock_runtime` where the process binary's
/// `main` function is. The process binary's `main` function must have the
/// signature `FnOnce() -> T`, where T is some concrete type that implements
/// `libtock_platform::Termination`.
///
/// # Example
/// ```
/// libtock_runtime::set_main!{main};
///
/// fn main() -> () { /* Omitted */ }
/// ```
// set_main! generates a function called `libtock_unsafe_main`, which is called
// by `rust_start`. The function has `unsafe` in its name because implementing
// it is `unsafe` (it *must* have the signature `libtock_unsafe_main() -> !`),
// but there is no way to enforce the use of `unsafe` through the type system.
// This function calls the client-provided function, which enforces its type
// signature.
#[macro_export]
macro_rules! set_main {
    {$name:ident} => {
        #[no_mangle]
        fn libtock_unsafe_main() -> ! {
            use $crate::TockSyscalls;
            #[allow(unreachable_code)] // so that fn main() -> ! does not produce a warning.
            $crate::startup::handle_main_return($name())
        }
    }
}

/// Executables must specify their stack size by using the `stack_size!` macro.
/// It takes a single argument, the desired stack size in bytes. Example:
/// ```
/// stack_size!{0x400}
/// ```
// stack_size works by putting a symbol equal to the size of the stack in the
// .stack_buffer section. The linker script uses the .stack_buffer section to
// size the stack. flash.sh looks for the symbol by name (hence #[no_mangle]) to
// determine the size of the stack to pass to elf2tab.
#[macro_export]
macro_rules! stack_size {
    {$size:expr} => {
        #[no_mangle]
        #[link_section = ".stack_buffer"]
        pub static mut STACK_MEMORY: [u8; $size] = [0; $size];
    }
}

pub fn handle_main_return<T: Termination>(result: T) -> ! {
    Termination::complete::<TockSyscalls>(result)
}

// rust_start is the first Rust code to execute in the process. It is called
// from start, which is written directly in assembly.
#[no_mangle]
extern "C" fn rust_start() -> ! {
    // TODO: Call memop() to inform the kernel of the stack and heap sizes +
    // locations. Also, perhaps we should support calling a heap initialization
    // function?

    extern "Rust" {
        fn libtock_unsafe_main() -> !;
    }
    unsafe {
        libtock_unsafe_main();
    }
}
