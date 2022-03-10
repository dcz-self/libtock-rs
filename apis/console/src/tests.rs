use super::*;
use libtock_platform::ErrorCode;
use libtock_unittest::{command_return, fake, ExpectedSyscall};

type Console = super::Console<fake::Syscalls>;

#[test]
fn no_driver() {
    let _kernel = fake::Kernel::new();
    assert!(!Console::driver_check());
}

#[test]
fn driver_check() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);

    assert!(Console::driver_check());
    assert_eq!(driver.take_bytes(), &[]);
}

#[test]
fn write_bytes_all() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);

    Console::write_all("foo".as_bytes());
    Console::write_all("bar".as_bytes());
    assert_eq!(
        driver.take_bytes(),
        "foobar".as_bytes(),
    );
}

#[test]
fn write_str() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);

    write!(Console, "foo").unwrap();
    assert_eq!(driver.take_bytes(), "foo".as_bytes())]);
}

/*
#[test]
fn failed_print() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: PRINT_1,
        argument0: 72,
        argument1: 0,
        override_return: Some(command_return::failure(ErrorCode::Fail)),
    });

    // The error is explicitly silenced, and cannot be detected.
    Console::print_1(72);

    // The fake driver still receives the command even if a fake error is injected.
    assert_eq!(driver.take_messages(), [fake::Message::Print1(72)]);
}*/
