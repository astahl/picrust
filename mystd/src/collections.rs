
pub mod bunch;
pub mod line;
pub mod ring;
pub mod sync_ring;
pub mod rectangular;

pub trait Sliceable<T> {
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
}
impl<T, const N: usize> Sliceable<T> for [T; N] {
    #[inline]
    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

impl<T> Sliceable<T> for [T] {
    #[inline]
    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

impl<T> Sliceable<T> for &mut [T] {
    #[inline]
    fn as_slice(&self) -> &[T] {
        self.as_ref()
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

#[derive(Debug)]
pub enum BufferError {
    Overflow { write_index: usize },
    OutOfRange { read_index: usize },
    IndexAlreadyFreed { index: usize },
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
