//! An extremely simple libtock-rs example. Just turns on all the LEDs.

#![no_main]
#![no_std]

use libtock2::alarm::{Alarm, Milliseconds};
use libtock2::leds::Leds;
use libtock2::runtime::{set_main, stack_size};

set_main! {main}
stack_size! {0x200}

fn main() {
    if let Ok(leds_count) = Leds::count() {
        loop {
            for led_index in 0..leds_count {
                let _ = Leds::toggle(led_index as u32);
            }
            Alarm::sleep_for(Milliseconds(250)).unwrap();
        }
    }
}
