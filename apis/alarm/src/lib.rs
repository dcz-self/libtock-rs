#![no_std]

use libtock_platform as platform;
use libtock_platform::{
    DefaultConfig, ErrorCode, Syscalls,
};
use libtock_platform::share;

/// The alarm driver
///
/// # Example
/// ```ignore
/// use libtock2::Alarm;
///
/// // Wait for timeout
/// Alarm::sleep(Alarm::Milliseconds(2500));
/// ```

pub struct Alarm<S: Syscalls, C: platform::subscribe::Config = DefaultConfig>(S, C);

#[derive(Copy, Clone)]
pub struct Hz(pub u32);

pub trait Convert {
    fn to_ticks(self, freq: Hz) -> Ticks;
}

#[derive(Copy, Clone)]
pub struct Ticks(pub u32);

impl Convert for Ticks {
    fn to_ticks(self, _freq: Hz) -> Ticks {
        self
    }
}

#[derive(Copy, Clone)]
pub struct Milliseconds(pub u32);

impl Convert for Milliseconds {
    fn to_ticks(self, freq: Hz) -> Ticks {
        // Saturating multiplication will top out at about 1 hour at 1MHz.
        // It's large enough for an alarm, and much simpler than failing
        // or losing precision for short sleeps.
        Ticks(self.0.saturating_mul(freq.0) / 1000)
    }
}

impl<S: Syscalls, C: platform::subscribe::Config> Alarm<S, C> {
    /// Run a check against the console capsule to ensure it is present.
    ///
    /// Returns number of concurrent notifications supported,
    /// 0 if unbounded.
    #[inline(always)]
    pub fn driver_check() -> Result<u32, ErrorCode> {
        S::command(DRIVER_NUM, command::DRIVER_CHECK, 0, 0).to_result()
    }

    pub fn get_frequency() -> Result<Hz, ErrorCode> {
        S::command(DRIVER_NUM, command::FREQUENCY, 0, 0).to_result().map(Hz)
    }

    pub fn sleep_for<T: Convert>(time: T) -> Result<(), ErrorCode> {
        let freq = Self::get_frequency()?;
        let ticks = time.to_ticks(freq);
        
        let called = core::cell::Cell::new(Option::<(u32, u32)>::None);
        share::scope(|subscribe| {
            S::subscribe::<_, _, C, DRIVER_NUM, { subscribe::CALLBACK }>(subscribe, &called)?;

            S::command(DRIVER_NUM, command::SET_RELATIVE, ticks.0, 0).to_result().map(|_when: u32| ())?;

            loop {
                S::yield_wait();
                if let Some((_when, _ref)) = called.get() {
                    return Ok(());
                }
            }
        })
    }
}

//#[cfg(test)]
//mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 0;

// Command IDs
#[allow(unused)]
mod command {
    pub const DRIVER_CHECK: u32 = 0;
    pub const FREQUENCY: u32 = 1;
    pub const TIME: u32 = 2;
    pub const STOP: u32 = 3;
    
    pub const SET_RELATIVE: u32 = 5;
    pub const SET_ABSOLUTE: u32 = 6;
    
}

#[allow(unused)]
mod subscribe {
    pub const CALLBACK: u32 = 0;
}