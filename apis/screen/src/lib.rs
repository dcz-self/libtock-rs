#![no_std]

use libtock_platform as platform;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;
use libtock_platform::{DefaultConfig, ErrorCode, Syscalls};

pub mod lightness {
    pub const OFF: u32 = 0;
    pub const MIN: u32 = 1;
    pub const MAX: u32 = 65536;
}

#[derive(Copy, Clone, Debug)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum PixelFormat {
    /// Pixels encoded as 1-bit, used for monochromatic displays
    Mono = 0,
    /// Pixels encoded as 2-bit red channel, 3-bit green channel, 3-bit blue channel.
    RGB_233 = 1,
    /// Pixels encoded as 5-bit red channel, 6-bit green channel, 5-bit blue channel.
    RGB_565 = 2,
    /// Pixels encoded as 8-bit red channel, 8-bit green channel, 8-bit blue channel.
    RGB_888 = 3,
    /// Pixels encoded as 8-bit alpha channel, 8-bit red channel, 8-bit green channel, 8-bit blue channel.
    ARGB_8888 = 4,
}

impl PixelFormat {
    /// Bits per pixel
    pub fn bpp(self) -> u8 {
        match self {
            PixelFormat::Mono => 1,
            PixelFormat::RGB_233 => 8,
            PixelFormat::RGB_565 => 16,
            PixelFormat::RGB_888 => 24,
            PixelFormat::ARGB_8888 => 32,
        }
    }
}

impl TryFrom<u32> for PixelFormat {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, ()> {
        use PixelFormat::*;
        match v {
            0 => Ok(Mono),
            1 => Ok(RGB_233),
            2 => Ok(RGB_565),
            3 => Ok(RGB_888),
            4 => Ok(ARGB_8888),
            _ => Err(()),
        }
    }
}

/// The screen driver.
///
/// It allows libraries to control an attached display module.
///
/// # Example
/// ```ignore
/// use libtock2::Screen;
///
/// // Writes "foo", followed by a newline, to the console
/// let mut writer = Console::writer();
/// writeln!(writer, foo).unwrap();
/// ```
pub struct Screen<
    S: Syscalls,
    C: platform::allow_ro::Config + platform::subscribe::Config = DefaultConfig,
>(S, C);

impl<S: Syscalls, C: platform::allow_ro::Config + platform::subscribe::Config> Screen<S, C> {
    /// Run a check against the console capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn driver_check() -> bool {
        S::command(DRIVER_NUM, command::DRIVER_CHECK, 0, 0).is_success()
    }

    pub fn get_resolution() -> Result<Resolution, ErrorCode> {
        S::command(DRIVER_NUM, command::RESOLUTION, 0, 0)
            .to_result()
            .map(|(width, height): (u32, u32)| Resolution { width, height })
    }
    
    pub fn get_pixel_format() -> Result<PixelFormat, ErrorCode> {
        S::command(DRIVER_NUM, command::PIXEL_FORMAT, 0, 0)
            .to_result()
            .and_then(|v: u32|
                PixelFormat::try_from(v).map_err(|()| ErrorCode::Invalid)
            )
    }
    
    pub fn set_power(on: bool) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope(|subscribe| {
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMMANDS }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::SET_POWER, on as u32, 0)
            .to_result()?;

            loop {
                S::yield_wait();
                if let Some((_result, command)) = called.get() {
                    if command == Callback::Ready as u32 {
                        return Ok(());
                    }
                }
            }
        })
    }
    
    pub fn set_brightness(lightness: u32) -> Result<(), ErrorCode> {
        S::command(DRIVER_NUM, command::SET_BRIGHTNESS, lightness, 0)
            .to_result()
    }

    pub fn set_frame(frame: Rectangle) -> Result<(), ErrorCode> {
        let reg1 = (frame.x as u32) << 16 | frame.y as u32;
        let reg2 = (frame.width as u32) << 16 | frame.height as u32;
        S::command(DRIVER_NUM, command::SET_FRAME, reg1, reg2)
            .to_result()
    }

    /// Writes the whole buffer of pixel data to selected frame.
    /// Does not check buffer alignment or bounds.
    pub fn write(buffer: &[u8]) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope::<
            (
                AllowRo<_, DRIVER_NUM, { allow_ro::WRITE }>,
                Subscribe<_, DRIVER_NUM, { subscribe::COMMANDS }>,
            ),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();

            S::allow_ro::<C, DRIVER_NUM, { allow_ro::WRITE }>(allow_ro, buffer)?;

            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::COMMANDS }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::WRITE, buffer.len() as u32, 0).to_result()?;

            loop {
                S::yield_wait();
                if let Some((_result, command)) = called.get() {
                    if command == Callback::WriteComplete as u32 {
                        return Ok(());
                    }
                }
            }
        })
    }
}

// #[cfg(test)]
// mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0x90001;

// Command IDs
#[allow(unused)]
mod command {
    pub const DRIVER_CHECK: u32 = 0;
    pub const SET_POWER: u32 = 1;
    pub const SET_BRIGHTNESS: u32 = 2;
    pub const RESOLUTION: u32 = 23;
    pub const PIXEL_FORMAT: u32 = 25;
    pub const SET_FRAME: u32 = 100;
    pub const WRITE: u32 = 200;
    
}

#[allow(unused)]
enum Callback {
    Ready = 0,
    WriteComplete = 1,
    CommandComplete = 2,
}

mod subscribe {
    pub const COMMANDS: u32 = 0;
}

mod allow_ro {
    pub const WRITE: u32 = 0;
}
