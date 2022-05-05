//! Screen API example.
//! Prints screen properties and displays patterns in corners:
//! top-left: black, top-right: white, bottom-left: vertical stripes

#![no_main]
#![no_std]

use core::cmp;
use core::fmt::Write;
use libtock2::console::Console;
use libtock2::runtime::{set_main, stack_size};
use libtock2::screen::{PixelFormat, Rectangle, Screen, lightness};

set_main! {main}
stack_size! {0x4000}

const BUFSIZE: usize = 16*16*32; // 8192 bytes max

fn main() {
    //Screen::set_brightness(lightness::MAX).unwrap();
    Screen::set_power(true).unwrap();

    writeln!(Console::writer(), "Current resolution:").unwrap();
    let res = Screen::get_resolution().unwrap();
    writeln!(Console::writer(), "{:?}", res).unwrap();
    writeln!(Console::writer(), "Pixel format:").unwrap();
    let px = Screen::get_pixel_format().unwrap();
    writeln!(Console::writer(), "{:?}", px).unwrap();
    
    // If width or height is odd, leave a stripe in the middle
    let halfwidth = res.width as u16 / 2;
    let halfheight = res.height as u16 / 2;
    // black top left
    {
        let buf = [0; BUFSIZE];
        let frame = Rectangle {
            x: 0,
            y: 0,
            width: halfwidth,
            height: halfheight,
        };
        Screen::set_frame(frame).unwrap();
        Screen::write(&buf).unwrap();
    }
    // white top right
    {
        let buf = [0xff; BUFSIZE];
        let frame = Rectangle {
            x: res.width as u16 - halfwidth,
            y: 0,
            width: halfwidth,
            height: halfheight,
        };
        Screen::set_frame(frame).unwrap();
        Screen::write(&buf).unwrap();
    }
    // striped bottom left
    {
        let mut buf = [0x0; BUFSIZE];

        let window_size = cmp::max(px.bpp() as usize, 8) / 8;
        let (even, odd) = if let PixelFormat::Mono = px {
            ([0b10101010; 1].as_slice(), [0b10101010; 1].as_slice())
        } else {
            ([0; 4].as_slice(), [0xff; 4].as_slice())
        };
        
        for (i, chunk) in buf.chunks_mut(window_size).enumerate() {
            let pattern
                = if i % 2 == 0 { even }
                else { odd };
            chunk.copy_from_slice(&pattern[..window_size]);
        }
        // This will cause checkerboard if half screen width is odd
        
        let frame = Rectangle {
            x: 0,
            y: res.height as u16 - halfheight,
            width: halfwidth,
            height: halfheight,
        };
        Screen::set_frame(frame).unwrap();
        Screen::write(&buf).unwrap();
    }
}
