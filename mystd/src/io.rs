use core::num::NonZeroUsize;

pub mod buffered_writer;

#[derive(Clone, Copy, Debug)]
pub enum Error {
    NoReceiver,
    InvalidData,
    WriteZero,
    Interrupted,
    UnexpectedEof,
    ReadBufferZeroLength,
    Unknown { err_code: i32 },
    WouldBlock,
    TimedOut,
    ConnectionAborted,
}

#[derive(Clone, Copy, Debug)]
pub enum Size {
    Eof,
    Num(core::num::NonZeroUsize),
}

impl Size {
    pub const fn from_usize(value: usize) -> Self {
        match value {
            0 => Self::Eof,
            n => Self::Num(unsafe { NonZeroUsize::new_unchecked(n) }),
        }
    }

    pub const fn to_usize(self) -> usize {
        match self {
            Size::Eof => 0,
            Size::Num(n) => n.get(),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<Size>;

    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(&buf) {
                Ok(Size::Eof) => return Err(Error::WriteZero),
                Ok(written) => buf = &buf[written.to_usize()..],
                Err(Error::Interrupted) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> self::Result<()> {
        struct Adapter<'a, T: ?Sized + 'a> {
            inner: &'a mut T,
            error: Result<()>,
        }

        impl<T: Write + ?Sized> core::fmt::Write for Adapter<'_, T> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(core::fmt::Error)
                    }
                }
            }
        }
        let mut output = Adapter {
            inner: self,
            error: Ok(()),
        };
        match core::fmt::write(&mut output, fmt) {
            Ok(_) => Ok(()),
            Err(_) => {
                // check if the error came from the underlying `Write` or not
                if output.error.is_err() {
                    output.error
                } else {
                    Err(Error::InvalidData)
                }
            }
        }
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

impl self::Write for &mut [u8] {
    fn write(&mut self, buf: &[u8]) -> Result<Size> {
        if self.is_empty() {
            return Ok(Size::Eof);
        }
        let count = self.len().min(buf.len());
        let (dst, tail) = core::mem::take(self).split_at_mut(count);
        dst.copy_from_slice(&buf[..count]);
        *self = tail;
        Ok(Size::from_usize(count))
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T: self::Write> self::Write for Option<T> {
    fn write(&mut self, buf: &[u8]) -> Result<Size> {
        match self {
            Some(writer) => writer.write(buf),
            None => Err(Error::NoReceiver),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            Some(writer) => writer.flush(),
            None => Err(Error::NoReceiver),
        }
    }
}

impl<T: self::Write> self::Write for &mut T {
    fn write(&mut self, buf: &[u8]) -> Result<Size> {
        (**self).write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        (**self).flush()
    }
}

impl self::Write for &mut dyn self::Write {
    fn write(&mut self, buf: &[u8]) -> Result<Size> {
        (**self).write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        (**self).flush()
    }
}

impl core::fmt::Write for dyn self::Write {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

pub struct SplitWriter<W1, W2>(Option<W1>, Option<W2>)
where
    W1: self::Write,
    W2: self::Write;

impl<W1, W2> SplitWriter<W1, W2>
where
    W1: self::Write,
    W2: self::Write,
{
    pub const fn empty() -> Self {
        Self(None, None)
    }

    pub const fn with_writers(w1: W1, w2: W2) -> Self {
        Self(Some(w1), Some(w2))
    }

    pub fn replace_first(&mut self, w1: W1) -> Option<W1> {
        self.0.replace(w1)
    }

    pub fn replace_second(&mut self, w2: W2) -> Option<W2> {
        self.1.replace(w2)
    }
}

impl<A, B> self::Write for SplitWriter<A, B>
where
    A: self::Write,
    B: self::Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<Size> {
        match self.0.write(buf) {
            Ok(written) => match self.1.write_all(&buf[..written.to_usize()]) {
                Ok(_) => Ok(written),
                Err(Error::NoReceiver) => Ok(written),
                Err(e) => Err(e),
            },
            Err(Error::NoReceiver) => self.1.write(buf),
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self.0.flush() {
            Ok(_) => match self.1.flush() {
                Ok(_) => Ok(()),
                Err(Error::NoReceiver) => Ok(()),
                Err(e) => Err(e),
            },
            Err(Error::NoReceiver) => self.1.flush(),
            Err(e) => Err(e),
        }
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<Size>;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut remaining = buf;
        while !remaining.is_empty() {
            match self.read(remaining) {
                Ok(Size::Eof) => return Err(Error::UnexpectedEof),
                Ok(written) => remaining = &mut remaining[written.to_usize()..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn bytes(self) -> Bytes<Self>
    where
        Self: Sized,
    {
        Bytes { reader: self }
    }
}

pub struct Bytes<T: Read> {
    reader: T,
}

impl<T: Read> Iterator for Bytes<T> {
    type Item = self::Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0_u8];
        match self.reader.read(&mut buf) {
            Ok(Size::Eof) => None,
            Ok(_) => Some(Ok(buf[0])),
            Err(e) => Some(Err(e)),
        }
    }
}

impl self::Read for &[u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<Size> {
        let self_len = self.len();
        if self_len == 0 {
            return Ok(Size::Eof);
        }
        let buf_len = buf.len();
        if buf_len == 0 {
            return Err(Error::ReadBufferZeroLength);
        }
        let count = self.len().min(buf.len());
        let (src, tail) = core::mem::take(self).split_at(count);
        // avoid overhead of memcopy for single-element copy
        match count {
            1 => buf[0] = src[0],
            2 => unsafe { *buf.as_mut_ptr().cast::<[u8; 2]>() = *src.as_ptr().cast::<[u8; 2]>() },
            3 => unsafe { *buf.as_mut_ptr().cast::<[u8; 3]>() = *src.as_ptr().cast::<[u8; 3]>() },
            4 => unsafe { *buf.as_mut_ptr().cast::<[u8; 4]>() = *src.as_ptr().cast::<[u8; 4]>() },
            _ => buf[..count].copy_from_slice(src),
        }
        *self = tail;
        Ok(Size::from_usize(count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_u8_works() {
        let arr: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let mut slice = arr.as_slice();

        let mut dst0: [u8; 1] = [0; 1];
        let mut dst1: [u8; 3] = [0; 3];
        let mut dst2: [u8; 10] = [0; 10];
        slice.read(&mut dst0).unwrap();
        slice.read(&mut dst1).unwrap();
        let mut slice = arr.as_slice();
        slice.read(&mut dst2).unwrap();
        assert_eq!([1], dst0);
        assert_eq!([2, 3, 4], dst1);
        assert_eq!([1, 2, 3, 4, 5, 6, 7, 8, 0, 0], dst2);
    }
}
