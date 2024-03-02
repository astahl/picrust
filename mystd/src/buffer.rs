use core::{fmt::Write, marker::PhantomData};

pub trait Sliceable<T> : AsRef<[T]> + AsMut<[T]>{
    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut()
    }

}
impl<T, const N: usize> Sliceable<T> for [T;N] {}
impl<T> Sliceable<T> for [T] {}
impl<T> Sliceable<T> for &mut [T] {}

#[derive(Debug)]
pub enum BufferError {
    Overflow { write_index: usize },
    OutOfRange { read_index: usize },
    IndexAlreadyFreed { index: usize },
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PowOfTwoWrapping<const N: usize> (usize);
impl<const N: usize> PowOfTwoWrapping<N> {
    const LEN: usize = N.next_power_of_two();
    const MASK: usize = Self::LEN - 1;

    pub const fn default() -> Self {
        Self(0)
    }

    pub const fn next(self) -> (bool, Self) {
        let mut i = self.0;
        i = i.wrapping_add(1);
        i &= Self::MASK;
        (i <= self.0, Self(i))
    }

    pub fn increment(&mut self) -> bool {
        let before = self.0;
        self.0 = self.0.wrapping_add(1);
        self.0 &= Self::MASK;
        self.0 <= before
    }

    pub const fn previous(self) -> (bool, Self) {
        let mut i = self.0;
        i = i.wrapping_sub(1);
        i &= Self::MASK;
        (i <= self.0, Self(i))
    }

    pub fn decrement(&mut self) -> bool {
        let before = self.0;
        self.0 = self.0.wrapping_sub(1);
        self.0 &= Self::MASK;
        self.0 >= before
    }

    pub const fn add(self, value: usize) -> (bool, Self) {
        let mut i = self.0;
        i = i.wrapping_add(value & Self::MASK);
        i &= Self::MASK;
        (i <= self.0, Self(i))
    }

    pub const fn sub(self, value: usize) -> (bool, Self) {
        let mut i = self.0;
        i = i.wrapping_sub(value & Self::MASK);
        i &= Self::MASK;
        (i <= self.0, Self(i))
    }

    pub const fn eq(self, rhs: Self) -> bool {
        self.0 == rhs.0
    }
}



pub struct Ring<T, S: Sliceable<T>, const N: usize> {
    read: PowOfTwoWrapping<N>,
    write: PowOfTwoWrapping<N>,
    data: S,
    phantom: PhantomData<T>
}

impl<T, S: Sliceable<T>, const N: usize> Ring<T, S, N> {    
    pub fn adapting(slice: S) -> Self {
        assert!(N.is_power_of_two(), "N must be a power of two, is N={N}");
        assert!(slice.as_slice().len() >= N, "Slice must be at least len of N={N}");
        Self { read: PowOfTwoWrapping::default(), write: PowOfTwoWrapping::default(), data: slice, phantom: PhantomData{} }
    }

    pub const fn is_full(&self) -> bool {
        self.write.next().1.eq(self.read)
    }

    pub const fn is_empty(&self) -> bool {
        self.write.eq(self.read)
    }

    pub const fn len(&self) -> usize {
        if self.write.0 >= self.read.0 {
            self.write.0 - self.read.0
        } else {
            self.write.0 + N - self.read.0
        }
    }

    pub const fn continuous_capacities(&self) -> (usize, usize) {
        if self.write.0 >= self.read.0 {
            (N - self.write.0 - 1, self.read.0)
        } else {
            (self.read.0 - self.write.0 - 1, 0)
        }
    }


    pub const fn total_capacity(&self) -> usize {
        let (a, b) = self.continuous_capacities();
        a + b
    }

    pub fn put(&mut self, value: T) -> Result<(), BufferError> {
        if self.is_full() {
            Err(BufferError::Overflow{write_index: self.write.0})
        } else {
            unsafe { self.put_unchecked(value) };
            Ok(())
        }
    }

    pub unsafe fn put_unchecked(&mut self, value: T) {
        self.data.as_mut_slice()[self.write.0] = value;
        self.write.increment();
    }

    pub fn put_clobbering(&mut self, value: T) {
        self.data.as_mut_slice()[self.write.0] = value;
        let next = self.write.next().1;
        // don't advance if we hit the read pointer
        self.write = if next == self.read { self.write } else { next }
    }

    pub fn copy_from_slice(&mut self, src: &[T]) -> Result<(), BufferError> where T: Copy {
        let src_len = src.len();
        if self.total_capacity() < src.len() {
            return Err(BufferError::Overflow { write_index: self.write.0 })
        }

        let (a, b) = self.continuous_capacities();
        let start = self.write.0;
        if src_len <= a {
            let dst = &mut self.data.as_mut_slice()[start..start + src_len];
            dst.copy_from_slice(src);
            self.write.0 += src_len;
        } else {
            let (src_a, src_b) = src.split_at(a);
            let dst_a = &mut self.data.as_mut_slice()[start..start + a];
            dst_a.copy_from_slice(src_a);
            let remaining_len = src_b.len();
            let dst_b = &mut self.data.as_mut_slice()[..remaining_len];
            dst_b.copy_from_slice(src_b);
            self.write.0 = remaining_len;
        }

        Ok(())
    }

    pub fn peek(&self) -> Option<&T> {
        if !self.is_empty() {
            Some(&self.data.as_slice()[self.read.0])
        } else {
            None
        }
    }

    pub fn pop(&mut self) -> Option<&T> {
        if !self.is_empty() {
            let result = Some(&self.data.as_slice()[self.read.0]);
            self.read.increment();
            result
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.read = self.write;
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        let slice = self.data.as_slice();
        if self.read <= self.write {
            (&slice[self.read.0..self.write.0], &[])
        } else {
            (
                &slice[self.read.0..],
                &slice[..self.write.0],
            )
        }
    }

    pub fn make_continuous(&mut self) -> &[T] {
        let len = self.len();
        self.data.as_mut_slice().rotate_left(self.read.0);
        self.read.0 = 0;
        self.write.0 = len;
        unsafe { self.data.as_slice().get_unchecked(..len) }
    }
}

pub type RingArray<T, const N: usize> = Ring<T, [T; N], N>;

impl<T: Default + Copy, const N: usize> RingArray<T, N> {
    pub fn new() -> Self {
        assert!(N.is_power_of_two());
        Self::adapting([T::default(); N])
    }
}

impl<S: Sliceable<u8>, const N: usize> core::fmt::Write for Ring<u8, S, N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.copy_from_slice(s.as_bytes()).map_err(|_| core::fmt::Error)
    }
}

impl<S: Sliceable<char>,const N: usize> core::fmt::Write for Ring<char, S, N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.chars()
            .try_for_each(|char| self.put(char).map_err(|_| core::fmt::Error))
    }
}

impl<S: Sliceable<u8>, const N: usize> core::fmt::Display for Ring<u8, S, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (a, b) = self.as_slices();
        let str_a = core::str::from_utf8(a).map_err(|_| core::fmt::Error)?;
        let str_b = core::str::from_utf8(b).map_err(|_| core::fmt::Error)?;
        f.write_str(str_a)?;
        f.write_str(str_b)
    }
}

impl<S: Sliceable<char>, const N: usize> core::fmt::Display for Ring<char, S, N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (a, b) = self.as_slices();
        for c in a.into_iter().chain(b) {
            f.write_char(*c)?;
        }
        Ok(())
    }
}

pub struct Line<T, S: Sliceable<T>, const N: usize> {
    cursor: usize,
    end: usize,
    data: S,
    phantom: PhantomData<T>
}

impl<T, S: Sliceable<T>, const N: usize> Line<T, S, N> {
    pub fn adapting(buffer: S) -> Self {
        Self { cursor: 0, end: 0, data: buffer, phantom: PhantomData{} }
    }

    pub fn is_full(&self) -> bool {
        N == self.end
    }

    pub fn is_empty(&self) -> bool {
        0 == self.end
    }

    pub fn push_back(&mut self, value: T) -> Result<(), BufferError> {
        if self.is_full() {
            Err(BufferError::Overflow{write_index: self.end})
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

pub fn collect_into_array<T, const N: usize, I>(iter: I, fill: T) -> [T; N]
where
    T: Copy,
    I: Iterator<Item = T>,
{
    let mut result = [fill; N];
    for (dst, src) in result.iter_mut().zip(iter.take(N)) {
        *dst = src;
    }
    result
}


#[cfg(test)]
mod tests {
    use core::fmt::Write;

    use super::*;

    #[test]
    fn ring_array_new_works() {
        RingArray::<u8, 16>::new();
    }

    #[test]
    fn ring_adapting_slice_works() {
        let mut buffer = [0; 24];
        Ring::<_, _, 16>::adapting(buffer.as_mut_slice());
    }

    #[test]
    #[should_panic]
    fn ring_adapting_assert_minimum_len_works() {
        let mut buffer = [0; 15];
        Ring::<_, _, 16>::adapting(buffer.as_mut_slice());
    }

    #[test]
    #[should_panic]
    fn ring_adapting_assert_pow_of_two_works() {
        let mut buffer = [0; 16];
        Ring::<_, _, 9>::adapting(buffer.as_mut_slice());
    }

    #[test]
    fn ring_len_and_capacities_works() -> Result<(), BufferError> {
        let mut ring: RingArray::<u8, 4> = RingArray::new();
        assert_eq!(0, ring.len());
        assert_eq!((3,0), ring.continuous_capacities());
        ring.put(1)?;
        assert_eq!(1, ring.len());
        assert_eq!((2,0), ring.continuous_capacities());
        ring.put(2)?;
        assert_eq!(2, ring.len());
        assert_eq!((1,0), ring.continuous_capacities());
        ring.put(3)?;
        assert_eq!(3, ring.len());
        assert_eq!((0,0), ring.continuous_capacities());
        let _ = ring.pop();
        assert_eq!(2, ring.len());
        assert_eq!((0,1), ring.continuous_capacities());
        let _ = ring.pop();
        assert_eq!(1, ring.len());
        assert_eq!((0,2), ring.continuous_capacities());
        ring.put(4)?;
        ring.put(5)?;
        assert_eq!(3, ring.len());
        assert_eq!((0,0), ring.continuous_capacities());
        let _ = ring.pop();
        assert_eq!(2, ring.len());
        assert_eq!((1,0), ring.continuous_capacities());

        let _ = ring.pop();
        let _ = ring.pop();
        assert_eq!(0, ring.len());
        assert_eq!((2,1), ring.continuous_capacities());
        ring.make_continuous();
        assert_eq!(0, ring.len());
        assert_eq!((3,0), ring.continuous_capacities());
        Ok(())
    }

    #[test]
    fn ring_as_slice_works() -> Result<(), BufferError>{
        let mut ring: RingArray::<u8, 8> = RingArray::new();
        for i in 1..=7 {
            ring.put(i)?;
        }

        assert_eq!(([1_u8,2,3,4,5,6, 7].as_slice(), [].as_slice()), ring.as_slices());

        ring.pop();
        ring.put(8)?;

        assert_eq!(([2,3,4,5,6, 7, 8].as_slice(), [].as_slice()), ring.as_slices());
        ring.pop();
        ring.put(9)?;

        assert_eq!(([3_u8,4,5,6,7,8].as_slice(), [9_u8].as_slice()), ring.as_slices());

        for _ in 0..6 {
            ring.pop();
        }

        assert_eq!(([9].as_slice(), [].as_slice()), ring.as_slices());
        ring.pop();

        assert_eq!(([].as_slice(), [].as_slice()), ring.as_slices());
        Ok(())
    }

    #[test]
    fn ring_make_continuous_works() -> Result<(), BufferError>{
        let mut ring: RingArray::<u8, 8> = RingArray::new();
        for i in 1..=7 {
            ring.put(i)?;
        }
        ring.pop();
        ring.put(8)?;
        ring.pop();
        ring.put(9)?;

        assert_eq!(([3,4,5,6,7,8].as_slice(), [9].as_slice()), ring.as_slices());
        assert_eq!([3,4,5,6,7,8,9], ring.make_continuous());
        assert_eq!(([3,4,5,6,7,8,9].as_slice(), [].as_slice()), ring.as_slices());
        Ok(())
    }

    #[test]
    fn ring_fmt_write_bytes() {
        let mut ring: RingArray::<u8, 8> = RingArray::new();
        write!(&mut ring, "🔔din").expect("write should work");
        assert_eq!("🔔din".as_bytes(), ring.make_continuous());
    }

    #[test]
    fn ring_fmt_display_bytes() {
        let mut ring: RingArray::<u8, 8> = RingArray::new();
        write!(&mut ring, "🔔din").expect("write should work");

        let mut buffer = RingArray::<u8, 16>::new();
        write!(&mut buffer, "{}", ring).expect("write to write");
        writeln!(&mut buffer, "{}", ring).expect("write to write");

        assert_eq!("🔔din🔔din\n".as_bytes(), buffer.make_continuous());
    }
}


pub struct Bunch<T, S: Sliceable<T>, const N: usize> {
    count_filled: usize,
    insert_index: usize,
    insert_map: [usize; N],
    data: S,
    phantom: PhantomData<T>
}

impl<T, S: Sliceable<T>, const N: usize> Bunch<T, S, N>   {
    pub fn adapting(buffer: S) -> Self {
        Self { count_filled: 0, insert_index: 0, insert_map: core::array::from_fn::<_, N, _>(|i| i + 1), data: buffer, phantom: PhantomData{} }
    }

    pub fn put(&mut self, value: T) -> Result<usize, BufferError>{
        let index = self.insert_index;
        if index == N {
            return Err(BufferError::Overflow { write_index: index })
        } 
        self.data.as_mut()[index] = value;
        self.insert_index = self.insert_map[index];
        self.insert_map[index] = 0;
        self.count_filled += 1;
        Ok(index)
    }

    pub fn take(&mut self, index: usize) -> Result<T, BufferError> where T: Copy {
        let copy = self.data.as_mut()[index];
        self.free(index)?;
        Ok(copy)
    }

    pub fn free(&mut self, index: usize) -> Result<(), BufferError> {
        if index >= N {
            return Err(BufferError::OutOfRange { read_index: index })
        } 
        if self.insert_map[index] != 0 {
            return Err(BufferError::IndexAlreadyFreed { index })
        }
        self.insert_map[index] = self.insert_index;
        self.insert_index = index;
        self.count_filled -= 1;
        Ok(())
    }
}