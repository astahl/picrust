use super::BufferError;
use super::Sliceable;
use core::marker::PhantomData;

pub struct Line<T, S: Sliceable<T>, const N: usize> {
    cursor: usize,
    end: usize,
    data: S,
    _phantom: PhantomData<T>,
}

impl<T, S: Sliceable<T>, const N: usize> Line<T, S, N> {
    pub fn adapting(buffer: S) -> Self {
        Self {
            cursor: 0,
            end: 0,
            data: buffer,
            _phantom: PhantomData {},
        }
    }

    pub fn is_full(&self) -> bool {
        N == self.end
    }

    pub fn is_empty(&self) -> bool {
        0 == self.end
    }

    pub fn push_back(&mut self, value: T) -> Result<(), BufferError> {
        if self.is_full() {
            Err(BufferError::Overflow {
                write_index: self.end,
            })
        } else {
            self.cursor = self.end;
            self.data.as_mut_slice()[self.cursor] = value;
            self.end += 1;
            Ok(())
        }
    }

    pub fn pop_back(&mut self) -> Option<&T> {
        if !self.is_empty() {
            self.end -= 1;
            let result = Some(&self.data.as_slice()[self.end]);
            result
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.end = 0;
        self.cursor = 0;
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { self.data.as_ref().get_unchecked(..self.end) }
    }
}

pub type LineArray<T, const N: usize> = Line<T, [T; N], N>;

impl<T: Default + Copy, const N: usize> LineArray<T, N> {
    pub fn new() -> Self {
        Self::adapting([T::default(); N])
    }
}
