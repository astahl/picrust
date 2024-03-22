use core::{ops::Index, ptr::{slice_from_raw_parts, slice_from_raw_parts_mut}};

use crate::slice::slice2d::{from_raw_parts_mut, MutSlice2d, Slice2d};

use super::Sliceable;


impl<T, const M: usize, const N: usize> Sliceable<T> for [[T; M]; N] {
    fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.as_ptr().cast(), M * N) }
    }

    fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), M * N) }
    }
}


pub struct Rectangular<T, S: Sliceable<T>, const W: usize, const P: usize, const H: usize> {
    data: S,
    _phantom: core::marker::PhantomData<T>,
}

impl<T, S: Sliceable<T>, const W: usize, const P: usize, const H: usize> Rectangular<T, S, W, P, H> {
    const _PITCH_CHECK: usize = H / P / W;

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

    pub fn as_slice2d(&self) -> Slice2d<T> {
        unsafe { crate::slice::slice2d::from_raw_parts(self.data.as_slice().as_ptr(), W, P, H) }
    }

    pub fn as_mut_slice2d(&mut self) -> MutSlice2d<T> {
        unsafe { crate::slice::slice2d::from_raw_parts_mut(self.data.as_mut_slice().as_mut_ptr(), W, P, H) }
    }
}


// impl<T, S: Sliceable<T>, const W: usize, const P: usize, const H: usize, I: Index2d> core::ops::Index<I> for Rectangular<T,S,W,P,H> {
//     type Output = T;

//     fn index(&self, index: I) -> &Self::Output {
//         let x = index.x();
//         let y = index.y();
//         assert!(x < W, "2D Index x={x} out of bounds, width is {W}");
//         assert!(y < H, "2D Index y={y} out of bounds, height is {H}");
//         unsafe { self.get_unchecked(x, y) }
//     }
// }

// impl<T, S: Sliceable<T>, const W: usize, const P: usize, const H: usize, I: Index2d> core::ops::IndexMut<I> for Rectangular<T,S,W,P,H> {
//     fn index_mut(&mut self, index: I) -> &mut Self::Output {
//         let x = index.x();
//         let y = index.y();
//         assert!(x < W, "2D Index x={x} out of bounds, width is {W}");
//         assert!(y < H, "2D Index y={y} out of bounds, height is {H}");
//         unsafe { self.get_unchecked_mut(x, y) }
//     }
// }

pub type RectangularArray<T, const W: usize, const H: usize> = Rectangular<T, [[T; W]; H], W, W, H>;

impl<T: Default + Copy, const W: usize, const H: usize> RectangularArray<T, W, H> {
    pub fn new() -> Self {
        Self::adapting([[T::default(); W]; H])
    }
}

pub fn from_fn<T, const W: usize, const H: usize, F: Fn(usize, usize) -> T>(cb: F) -> RectangularArray<T, W, H> {
    RectangularArray::adapting(core::array::from_fn(|h| core::array::from_fn(|w| cb(w,h))))
}

macro_rules! arr2d {
    () => {
        RectangularArray::adapting([[1,2,3,4], [1,2,3,2]])   
    };
    ($([$($val:expr),+]),+) => {
        RectangularArray::adapting([
            $([ $( $val, )+ ],)+
        ])   
    };
}


#[cfg(test)]
mod tests {
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
        *rect.as_mut_slice2d().get_mut((1,2)).unwrap() = 3;
    }
}