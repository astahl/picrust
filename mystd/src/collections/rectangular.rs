
use crate::slice::slice2d::traits::{self, DebugSlice2dTrait, MutSlice2dTrait, Slice2dTrait};

use super::{MutSliceable, Sliceable};


pub struct Rectangular<T, S, const W: usize, const P: usize, const H: usize> 
where S: Sliceable<T> {
    data: S,
    _phantom: core::marker::PhantomData<T>,
}


impl<T, S, const W: usize, const P: usize, const H: usize> 
Rectangular<T, S, W, P, H> 
where S: Sliceable<T> {
    pub fn adapting(buffer: S) -> Self {
        Self {
            data: buffer,
            _phantom: core::marker::PhantomData {},
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        if x < W && y < H {
            Some(unsafe { self.get_unchecked(x, y) })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> &T {
        self.data.as_slice().get_unchecked(x + y * P)
    }

    
}
impl<T, S, const W: usize, const P: usize, const H: usize> 
Rectangular<T, S, W, P, H> 
where S: MutSliceable<T> {
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < W && y < H {
            Some(unsafe { self.get_unchecked_mut(x, y) })
        } else {
            None
        }
    }

    pub unsafe fn get_unchecked_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.data.as_mut_slice().get_unchecked_mut(x + y * P)
    }
}


impl<T, S: Sliceable<T>, const W: usize, const P: usize, const H: usize> Slice2dTrait 
for Rectangular<T, S, W, P, H> {
    type Element = T;

    #[inline]
    fn as_ptr(&self) -> *const Self::Element {
        self.data.as_slice().as_ptr()
    }

    #[inline]
    fn width(&self) -> usize {
        W
    }

    #[inline]
    fn height(&self) -> usize {
        H
    }

    #[inline]
    fn pitch(&self) -> usize {
        P
    }
}

impl<T, S: MutSliceable<T>, const W: usize, const P: usize, const H: usize> MutSlice2dTrait 
for Rectangular<T, S, W, P, H> {
    fn as_mut_ptr(&mut self) -> *mut Self::Element {
        self.data.as_mut_slice().as_mut_ptr()
    }
}



impl<T, S, const W: usize, const P: usize, const H: usize> traits::DebugSlice2dTrait 
for Rectangular<T, S, W, P, H>
where 
    T: core::fmt::Debug, 
    S: Sliceable<T>
{}

impl<T, S, const W: usize, const P: usize, const H: usize> core::fmt::Debug 
for Rectangular<T, S, W, P, H> 
where 
    T: core::fmt::Debug, 
    S: Sliceable<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt_debug(f)
    }
}

impl<R, T, S, const W: usize, const P: usize, const H: usize> traits::PartialEqSlice2dTrait<R> 
for Rectangular<T, S, W, P, H>
where 
    R: Slice2dTrait,
    T: PartialEq<R::Element>,
    S: Sliceable<T>
{}

impl<R, T, S, const W: usize, const P: usize, const H: usize> core::cmp::PartialEq<R> 
for Rectangular<T, S, W, P, H>
where 
    R: Slice2dTrait,
    T: PartialEq<R::Element>,
    S: Sliceable<T>
{
    fn eq(&self, other: &R) -> bool {
        use traits::PartialEqSlice2dTrait;
        self.cmp_eq(other)
    }
}


impl<T, const M: usize, const N: usize> Sliceable<T> for [[T; M]; N] {
    fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr().cast(), M * N) }
    }
}
impl<T, const M: usize, const N: usize> MutSliceable<T> for [[T; M]; N] {
    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), M * N) }
    }
}


pub type RectangularArray<T, const W: usize, const H: usize> = Rectangular<T, [[T; W]; H], W, W, H>;

impl<T: Default + Copy, const W: usize, const H: usize> RectangularArray<T, W, H> {
    pub fn new() -> Self {
        Self::adapting([[T::default(); W]; H])
    }
}

pub fn from_fn<T, const W: usize, const H: usize, F: Fn(usize, usize) -> T>(cb: F) -> RectangularArray<T, W, H> {
    RectangularArray::adapting(core::array::from_fn(|h| core::array::from_fn(|w| cb(w,h))))
}

#[macro_export]
macro_rules! arr2d {
    ($([$($val:expr),+]),+) => {
        $crate::collections::rectangular::RectangularArray::adapting([
            $([ $( $val, )+ ],)+
        ])   
    };
}


#[cfg(test)]
mod tests {
    use crate::{collections::ring::RingArray, slice::slice2d::traits::Slice2dTrait};

    use super::*;

    #[test]
    fn simple_access() {
        let mut rect = RectangularArray::<u8, 10, 10>::new();
        *rect.get_mut(5, 5).unwrap() = 5;
        assert_eq!(5, *rect.get(5, 5).unwrap());
    }

    // #[test]
    // #[should_panic]
    // fn checked_access() {
    //     let rect = RectangularArray::<u8, 10, 10>::new();
    //     assert_eq!(0, rect[(9, 9)]);
    //     assert_eq!(0, rect[(10, 8)]);
    // }

    #[test]
    fn macro_works() {
        let rect = arr2d!([1, 2, 3], [1, 2+5, 3]);
        assert_eq!(7, *rect.get(1, 1).unwrap());
    }


    #[test]
    fn as_slice() {
        let mut rect = arr2d!([1,2,3], [1,2,3]);
        *rect.as_mut_slice2d().get_mut((1,0)).unwrap() = 5;
        assert_eq!(5, rect.as_slice2d()[(1,0)]);
        rect.as_mut_slice2d()[(0,0)] = 4;
        assert_eq!(4, unsafe { *rect.as_slice2d().get_unchecked((0,0)) });
        let slice = rect.as_mut_slice2d();
        let (l,r) = slice.split_at_col(1);
        assert_eq!(l, arr2d!([4], [1]));
        assert_eq!(r, arr2d!([5,3], [2,3]));
    }

    #[test]
    fn fmt_debug() {
        use core::fmt::Write;
        let rect = arr2d!([9,3,2,1], [2,3,2,1], [3,1,2,72]);
        let mut buf: RingArray<u8, 256> = RingArray::new();
        write!(buf, "{:#?}", rect).unwrap();
        assert_eq!(
"[
    [9, 3, 2, 1],
    [2, 3, 2, 1],
    [3, 1, 2, 72],
]", 
            buf.to_str().unwrap())
    }
}