use core::str;

use crate::{uDebug, uDisplay, uWrite, Formatter};

macro_rules! ixx {
    ($uxx:ty, $n:expr, $buf:expr) => {{
        let n = $n;
        let negative = n.is_negative();
        let mut n = if negative {
            match n.checked_abs() {
                Some(n) => n as $uxx,
                None => <$uxx>::max_value() / 2 + 1,
            }
        } else {
            n as $uxx
        };
        let mut i = $buf.len() - 1;
        loop {
            *$buf
                .get_mut(i)
                .unwrap_or_else(|| unsafe { assume_unreachable!() }) = (n % 10) as u8 + b'0';
            n = n / 10;

            if n == 0 {
                break;
            } else {
                i -= 1;
            }
        }

        if negative {
            i -= 1;
            *$buf
                .get_mut(i)
                .unwrap_or_else(|| unsafe { assume_unreachable!() }) = b'-';
        }

        unsafe { str::from_utf8_unchecked($buf.get(i..).unwrap_or_else(|| assume_unreachable!())) }
    }};
}

fn isize(n: isize, buf: &mut [u8]) -> &str {
    ixx!(usize, n, buf)
}

impl uDebug for i8 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex {
            <u8 as uDebug>::fmt(&(*self as u8), f)
        } else {
            let mut buf: [u8; 4] = [0; 4];

            f.write_str(isize(isize::from(*self), &mut buf))
        }
    }
}

impl uDisplay for i8 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i8 as uDebug>::fmt(self, f)
    }
}

impl uDebug for i16 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex {
            <u16 as uDebug>::fmt(&(*self as u16), f)
        } else {
            let mut buf: [u8; 6] = [0; 6];

            f.write_str(isize(isize::from(*self), &mut buf))
        }
    }
}

impl uDisplay for i16 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i16 as uDebug>::fmt(self, f)
    }
}

impl uDebug for i32 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex {
            <u32 as uDebug>::fmt(&(*self as u32), f)
        } else {
            let mut buf: [u8; 11] = [0; 11];

            f.write_str(isize(*self as isize, &mut buf))
        }
    }
}

impl uDisplay for i32 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i32 as uDebug>::fmt(self, f)
    }
}

impl uDebug for i64 {
    #[cfg(any(target_pointer_width = "32", target_pointer_width = "16"))]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex {
            <u64 as uDebug>::fmt(&(*self as u64), f)
        } else {
            let mut buf: [u8; 20] = [0; 20];

            let s = ixx!(u64, *self, buf);
            f.write_str(s)
        }
    }

    #[cfg(target_pointer_width = "64")]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex {
            <u64 as uDebug>::fmt(&(*self as u64), f)
        } else {
            let mut buf: [u8; 20] = [0; 20];

            f.write_str(isize(*self as isize, &mut buf))
        }
    }
}

impl uDisplay for i64 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i64 as uDebug>::fmt(self, f)
    }
}

impl uDebug for i128 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        if f.hex {
            <u128 as uDebug>::fmt(&(*self as u128), f)
        } else {
            let mut buf: [u8; 40] = [0; 40];

            let s = ixx!(u128, *self, buf);
            f.write_str(s)
        }
    }
}

impl uDisplay for i128 {
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i128 as uDebug>::fmt(self, f)
    }
}

impl uDebug for isize {
    #[cfg(target_pointer_width = "16")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i16 as uDebug>::fmt(&(*self as i16), f)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i32 as uDebug>::fmt(&(*self as i32), f)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i64 as uDebug>::fmt(&(*self as i64), f)
    }
}

impl uDisplay for isize {
    #[cfg(target_pointer_width = "16")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i16 as uDisplay>::fmt(&(*self as i16), f)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i32 as uDisplay>::fmt(&(*self as i32), f)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline(always)]
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <i64 as uDisplay>::fmt(&(*self as i64), f)
    }
}
