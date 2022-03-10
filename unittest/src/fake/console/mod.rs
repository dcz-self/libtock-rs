//! Fake implementation of the Console API, documented here:
//! https://github.com/tock/tock/blob/master/doc/syscalls/00001_console.md
//!
//! Like the real API, `Console` prints each message it is commanded to
//! print. It also keeps a log of the messages as a byte slice, which can
//! be retrieved via `take_bytes` for use in unit tests.

use core::cell::Cell;
use libtock_platform::{CommandReturn, ErrorCode};

use crate::upcall;
use crate::RoAllowBuffer;

pub struct Console {
    messages: Cell<Vec<u8>>,
    buffer: Cell<RoAllowBuffer>,
}

impl Console {
    pub fn new() -> std::rc::Rc<Console> {
        std::rc::Rc::new(Console {
            messages: Default::default(),
            buffer: Default::default(),
        })
    }

    /// Returns the bytes that have been submitted so far,
    /// and clears them.
    pub fn take_bytes(&self) -> Vec<u8> {
        self.messages.take()
    }
}

impl crate::fake::SyscallDriver for Console {
    fn id(&self) -> u32 {
        DRIVER_NUM
    }
    fn num_upcalls(&self) -> u32 {
        2
    }

    fn allow_readonly(
        &self,
        buffer_num: u32,
        buffer: RoAllowBuffer,
    ) -> Result<RoAllowBuffer, (RoAllowBuffer, ErrorCode)> {
        if buffer_num == 1 {
            Ok(self.buffer.replace(buffer))
        } else {
            Err((buffer, ErrorCode::Invalid))
        }
    }

    fn command(&self, command_num: u32, _argument0: u32, _argument1: u32) -> CommandReturn {
        match command_num {
            DRIVER_CHECK => {}
            WRITE => {
                let mut bytes = self.messages.take();
                let buffer = self.buffer.take();
                bytes.extend_from_slice(&*buffer);
                let size = buffer.len();
                self.buffer.set(buffer);
                self.messages.set(bytes);
                upcall::schedule(DRIVER_NUM, 1, (size as u32, 0, 0))
                    .expect("Unable to schedule upcall {}");
            }
            _ => return crate::command_return::failure(ErrorCode::NoSupport),
        }
        crate::command_return::success()
    }
}

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests;

const DRIVER_NUM: u32 = 1;

// Command numbers
const DRIVER_CHECK: u32 = 0;
const WRITE: u32 = 1;
//const READ: u32 = 2;
//const ABORT: u32 = 3;
