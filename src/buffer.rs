pub enum BufferError {
    Overflow
}


pub struct Ring<T> {
    read: u16,
    write: u16,
    data: [T; u16::MAX as usize],
}

impl<T> Ring<T> {
    pub const fn is_full(&self) -> bool {
        self.write.wrapping_add(1) == self.read
    }

    pub const fn is_empty(&self) -> bool {
        self.write == self.read
    }

    pub fn put(&mut self, value: T) -> Result<(), BufferError> {
        if self.is_full() {
            Err(BufferError::Overflow)
        } else {
            unsafe { self.put_unchecked(value) };
            Ok(())
        }
    }

    pub unsafe fn put_unchecked(&mut self, value: T) {
        self.data[self.write as usize] = value;
        self.write = self.write.wrapping_add(1);
    }

    pub fn put_clobbering(&mut self, value: T) {
        self.data[self.write as usize] = value;
        let next = self.write.wrapping_add(1);
        // don't advance if we hit the read pointer
        self.write = if next == self.read { self.write } else { next }
    }

    pub const fn peek(&self) -> Option<&T> {
        if !self.is_empty() {
            Some(&self.data[self.read as usize])
        } else {
            None
        }
    }

    pub fn pop(&mut self) -> Option<&T> {
        if !self.is_empty() {
            let result = Some(&self.data[self.read as usize]);
            self.read = self.read.wrapping_add(1);
            result
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.read = self.write;
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        if self.read <= self.write {
            (&self.data[self.read as usize .. self.write as usize], &[])
        } else {
            (&self.data[self.read as usize ..], &self.data[.. self.write as usize])
        }
    }
}

impl <T: Default + Copy> Ring<T> {
    pub fn new() -> Self {
        Self {
            read: 0,
            write: 0,
            data: [T::default(); u16::MAX as usize],
        }
    }

    pub fn default(length: u16) -> Self {
        Self {
            read: 0,
            write: length,
            data: [T::default(); u16::MAX as usize],
        }
    }
}




impl core::fmt::Write for Ring<u8> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.bytes().try_for_each(|byte| {
            self.put(byte).map_err(|_| core::fmt::Error)
        })
    }
}

impl core::fmt::Write for Ring<char> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars().try_for_each(|char| {
            self.put(char).map_err(|_| core::fmt::Error)
        })
    }
}