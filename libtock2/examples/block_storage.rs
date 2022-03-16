//! An extremely simple libtock-rs example. Just prints out a message
//! using the Console capsule, then terminates.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock2::console::Console;
use libtock2::block_storage::BlockStorage;
use libtock2::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x400}

fn main() {
    if BlockStorage::driver_check() {
        let g = BlockStorage::get_geometry();
        writeln!(Console::writer(), "Write block size: {} bytes", g.write_block_size).unwrap();
        writeln!(Console::writer(), "Erase block size: {} bytes", g.erase_block_size).unwrap();
    } else {
        writeln!(Console::writer(), "No block device detected").unwrap();
    }
}
