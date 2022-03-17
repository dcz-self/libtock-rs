//! An extremely simple libtock-rs example. Just prints out a message
//! using the Console capsule, then terminates.

#![no_main]
#![no_std]
use core::fmt::Write;
use libtock2::console::Console;
use libtock2::block_storage::BlockStorage;
use libtock2::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x1800}

fn main() {
    let mut w = Console::writer();
    if BlockStorage::driver_check() {
        let g = BlockStorage::get_geometry();
        writeln!(&mut w, "Write block size: {} bytes", g.write_block_size).unwrap();
        writeln!(Console::writer(), "Erase block size: {} bytes", g.erase_block_size).unwrap();
        let mut buf = [0; 4096];
        if g.write_block_size as usize > buf.len() {
            writeln!(Console::writer(), "Block size bigger than preallocated buffer, writes will be inaccurate.").unwrap();
        }
        BlockStorage::read(43, &mut buf).unwrap();
        writeln!(&mut w, "First bytes of sector 43: {:?}", &buf[..10]).unwrap();
        BlockStorage::erase(43).unwrap();
        BlockStorage::read(43, &mut buf).unwrap();
        writeln!(&mut w, "Erased sector to: {:?}", &buf[..10]).unwrap();
        buf = [0; 4096];
        buf[2] = 137;
        BlockStorage::write(43, &mut buf).unwrap();
        BlockStorage::read(43, &mut buf).unwrap();
        writeln!(&mut w, "Written sector: {:?}", &buf[..10]).unwrap();
    } else {
        writeln!(Console::writer(), "No block device detected").unwrap();
    }
}
