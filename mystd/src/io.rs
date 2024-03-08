
#[derive(Debug)]
pub enum Error {
    InvalidData,
    Interrupted,
    UnexpectedEof,
    Unknown{err_code: i32}
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut remaining = buf;
        while !remaining.is_empty() {
            match self.write(&remaining) {
                Ok(written) => remaining = remaining.split_at(written).1,
                Err(Error::Interrupted) => {},
                Err(e) => return Err(e),
            }
        }
        self.flush()
    }
}

impl self::Write for &mut [u8] {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let count = self.len().min(buf.len());
        let (dst, tail) = core::mem::take(self).split_at_mut(count);
        dst.copy_from_slice(&buf[..count]); 
        *self = tail;
        Ok(count)
    }

    fn flush(&mut self) -> Result<()> { Ok(()) }
}

impl core::fmt::Write for dyn self::Write {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_all(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut remaining = buf;
        while !remaining.is_empty() {
            match self.read(remaining) {
                Ok(0) => return Err(Error::UnexpectedEof),
                Ok(written) => remaining = remaining.split_at_mut(written).1,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn bytes(self) -> Bytes<Self> where Self: Sized {
        Bytes {
            reader: self
        }
    }
}

pub struct Bytes<T: Read> {
    reader: T
}

impl<T: Read> Iterator for Bytes<T> {
    type Item = self::Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [0_u8]; 
        match self.reader.read(&mut buf) {
            Ok(0) => None,
            Ok(_) => Some(Ok(buf[0])),
            Err(e) => Some(Err(e)),
        }
    }
}

impl self::Read for &[u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let count = self.len().min(buf.len());
        let (src, tail) = core::mem::take(self).split_at(count); 
        // avoid overhead of memcopy for single-element copy
        match count {
            1 => buf[0] = src[0],
            2 => unsafe { *buf.as_mut_ptr().cast::<[u8; 2]>() = *src.as_ptr().cast::<[u8; 2]>() },
            3 => unsafe { *buf.as_mut_ptr().cast::<[u8; 3]>() = *src.as_ptr().cast::<[u8; 3]>() },
            4 => unsafe { *buf.as_mut_ptr().cast::<[u8; 4]>() = *src.as_ptr().cast::<[u8; 4]>() },
            _ => buf[..count].copy_from_slice(src)
        }
        *self = tail;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_u8_works() {
        let arr: [u8; 8] = [1, 2, 3, 4,5,6,7,8];
        let mut slice = arr.as_slice();

        let mut dst0: [u8; 1] = [0; 1];
        let mut dst1: [u8; 3] = [0; 3];
        let mut dst2: [u8; 10] = [0; 10];
        slice.read(&mut dst0).unwrap();
        slice.read(&mut dst1).unwrap();
        let mut slice = arr.as_slice();
        slice.read(&mut dst2).unwrap();
        assert_eq!([1], dst0);
        assert_eq!([2,3,4], dst1);
        assert_eq!([1,2,3,4,5,6,7,8,0,0], dst2);
    }
}