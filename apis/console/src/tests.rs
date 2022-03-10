use super::*;
use core::fmt::Write;
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

    Console::write_all("foo".as_bytes()).unwrap();
    Console::write_all("bar".as_bytes()).unwrap();
    assert_eq!(driver.take_bytes(), "foobar".as_bytes(),);
}

#[test]
fn write_str() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);

    write!(Console::writer(), "foo").unwrap();
    assert_eq!(driver.take_bytes(), "foo".as_bytes());
}

#[test]
fn failed_print() {
    let kernel = fake::Kernel::new();
    let driver = fake::Console::new();
    kernel.add_driver(&driver);
    kernel.add_expected_syscall(ExpectedSyscall::Command {
        driver_id: DRIVER_NUM,
        command_id: command::WRITE,
        argument0: 5,
        argument1: 0,
        override_return: Some(command_return::failure(ErrorCode::Fail)),
    });

    assert_eq!(Console::write(&[0, 5]), Err(ErrorCode::Fail));
    assert_eq!(driver.take_bytes(), &[]);
}
