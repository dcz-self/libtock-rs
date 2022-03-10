use crate::fake;
use fake::console::*;

// Tests the command implementation.
#[test]
fn command() {
    use fake::SyscallDriver;
    let console = Console::new();
    assert!(console.command(DRIVER_CHECK, 1, 2).is_success());
    assert!(console.allow_readonly(1, RoAllowBuffer::default()).is_ok());
    assert!(console.allow_readonly(2, RoAllowBuffer::default()).is_err());
    // this requires kernel, so can't be unit tested
    //assert!(console.command(WRITE, 3, 4).is_success());
}

// Integration test that verifies Console works with fake::Kernel and
// libtock_platform::Syscalls.
#[test]
fn kernel_integration() {
    use libtock_platform::Syscalls;
    let kernel = fake::Kernel::new();
    let console = Console::new();
    kernel.add_driver(&console);
    assert!(fake::Syscalls::command(DRIVER_NUM, DRIVER_CHECK, 1, 2).is_success());
    assert!(fake::Syscalls::command(DRIVER_NUM, WRITE, 3, 4).is_success());
}
