#![no_std]

use core::fmt;
use libtock_platform::{Syscalls, ErrorCode};
use libtock_platform::allow_ro;
use libtock_platform::allow_ro::AllowRo;
use libtock_platform::share;
use libtock_platform::subscribe::Subscribe;

/// The console driver.
///
/// It allows libraries to pass strings to the kernel's console driver.
///
/// # Example
/// ```ignore
/// use libtock2::Console;
///
/// // Prints 0x45 and the app which called it.
/// Console::print(0x45);
/// ```
pub struct Console<S: Syscalls>(S);

impl<S: Syscalls> Console<S> {
    /// Run a check against the low-level debug capsule to ensure it is present.
    ///
    /// Returns `true` if the driver was present. This does not necessarily mean
    /// that the driver is working, as it may still fail to allocate grant
    /// memory.
    #[inline(always)]
    pub fn driver_check() -> bool {
        S::command(DRIVER_NUM, command::DRIVER_CHECK, 0, 0).is_success()
    }
    
    /// Writes bytes, returns count of bytes written.
    pub fn write(s: &[u8]) -> Result<u32, ErrorCode> { 
        let called = core::cell::Cell::new(Option::<(u32,)>::None);
        share::scope::<
            (AllowRo<_, DRIVER_NUM, 1>, Subscribe<_, DRIVER_NUM, 1>),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();
            
            S::allow_ro::<AllowConfig, DRIVER_NUM, 1>(allow_ro, s)?;
            
            S::subscribe::<
                _,
                _,
                subscribe::Config,
                DRIVER_NUM,
                1
                //subscribe::WRITE, // this confuses the compiler
            >(subscribe, &called)?;
            
            S::command(DRIVER_NUM, command::WRITE, s.len() as u32, 0)
                .to_result()?;
            
            loop {
                S::yield_wait();
                if let Some((bytes_read_count,)) = called.get() {
                    return Ok(bytes_read_count);
                }
            }
        })
    }
    
    /// Writes all bytes of a slice.
    /// This is an alternative to `fmt::Write::write`
    /// becaus this can actually return an error code.
    /// It's makes only one `subscribe` call,
    /// as opposed to calling `write` in a loop.
    fn write_all(s: &[u8]) -> Result<(), ErrorCode> {
        let called = core::cell::Cell::new(Option::<(u32,)>::None);
        share::scope::<
            (AllowRo<_, DRIVER_NUM, 1>, Subscribe<_, DRIVER_NUM, 1>),
            _,
            _,
        >(|handle| {
            let (allow_ro, subscribe) = handle.split();
            
            S::subscribe::<
                _,
                _,
                subscribe::Config,
                DRIVER_NUM,
                1
                //subscribe::WRITE, // this confuses the compiler
            >(subscribe, &called)?;
            
            let mut remaining = s.len();
            while remaining > 0 {
                S::allow_ro::<AllowConfig, DRIVER_NUM, 1>(
                    allow_ro,
                    &s[(s.len() - remaining)..],
                )?;
            
                S::command(DRIVER_NUM, command::WRITE, remaining as u32, 0)
                    .to_result()?;
                
                loop {
                    S::yield_wait();
                    if let Some((bytes_read_count,)) = called.get() {
                        remaining -= bytes_read_count as usize;
                        called.set(None);
                        break;
                    }
                }
            }
            Ok(())
        })
    }
}


impl<S: Syscalls> fmt::Write for Console<S> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        Self::write_all(s.as_bytes()).map_err(|_e| fmt::Error)
    }
}


struct AllowConfig;

// Not sure if nonzero returned buffers should receive special attention.
impl allow_ro::Config for AllowConfig {}

//#[cfg(test)]
//mod tests;

// -----------------------------------------------------------------------------
// Driver number and command IDs
// -----------------------------------------------------------------------------

const DRIVER_NUM: u32 = 1;

// Command IDs
#[allow(unused)]
mod command {
    pub const DRIVER_CHECK: u32 = 0;
    pub const WRITE: u32 = 1;
    pub const READ: u32 = 2;
    pub const ABORT: u32 = 3;
}

#[allow(unused)]
mod subscribe {
    use libtock_platform::subscribe;
    pub const WRITE: u32 = 1;
    pub const READ: u32 = 2;
    
    pub struct Config;
    
    impl subscribe::Config for Config {}
}