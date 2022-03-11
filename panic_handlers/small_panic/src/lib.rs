#![no_std]
use core::fmt::Write;
use libtock_console::Console;
use libtock_low_level_debug::{AlertCode, LowLevelDebug};
use libtock_platform::{ErrorCode, Syscalls};
use libtock_runtime::TockSyscalls;

/// This handler requires some 0x400 bytes of stack

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    let mut writer = Console::<TockSyscalls>::writer();
    if let Err(_) = writeln!(writer, "{}", info) {
        // Signal a panic using the LowLevelDebug capsule (if available).
        LowLevelDebug::<TockSyscalls>::print_alert_code(AlertCode::Panic);
    }
    // Exit with a non-zero exit code to indicate failure.
    // TODO(kupiakos@google.com): Make this logic consistent with tock/tock#2914
    // when it is merged.
    TockSyscalls::exit_terminate(ErrorCode::Fail as u32);
}
