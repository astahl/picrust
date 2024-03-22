use super::BufferError;
use super::Sliceable;
use core::marker::PhantomData;

pub struct Bunch<T, S: Sliceable<T>, const N: usize> {
    count_filled: usize,
    insert_index: usize,
    insert_map: [usize; N],
    data: S,
    _phantom: PhantomData<T>,
}

impl<T, S: Sliceable<T>, const N: usize> Bunch<T, S, N> {
    pub fn adapting(buffer: S) -> Self {
        Self {
            count_filled: 0,
            insert_index: 0,
            insert_map: core::array::from_fn::<_, N, _>(|i| i + 1),
            data: buffer,
            _phantom: PhantomData {},
        }
    }

    pub fn put(&mut self, value: T) -> Result<usize, BufferError> {
        let index = self.insert_index;
        if index == N {
            return Err(BufferError::Overflow { write_index: index });
        }
        self.data.as_mut_slice()[index] = value;
        self.insert_index = self.insert_map[index];
        self.insert_map[index] = 0;
        self.count_filled += 1;
        Ok(index)
    }

    pub fn take(&mut self, index: usize) -> Result<T, BufferError>
    where
        T: Copy,
    {
        let copy = self.data.as_mut_slice()[index];
        self.free(index)?;
        Ok(copy)
    }

    pub fn free(&mut self, index: usize) -> Result<(), BufferError> {
        if index >= N {
            return Err(BufferError::OutOfRange { read_index: index });
        }
        if self.insert_map[index] != 0 {
            return Err(BufferError::IndexAlreadyFreed { index });
        }
        self.insert_map[index] = self.insert_index;
        self.insert_index = index;
        self.count_filled -= 1;
        Ok(())
    }
}
