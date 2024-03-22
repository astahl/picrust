use core::ops::{Index, IndexMut};

pub unsafe trait Slice2dIndex<T: ?Sized>{
    type Output: ?Sized;

    fn get(self, slice2d: &T) -> Option<&Self::Output>;
    unsafe fn get_unchecked(self, slice2d: *const T) -> *const Self::Output;
    fn index(self, slice2d: &T) -> &Self::Output;
}

pub unsafe trait MutSlice2dIndex<T: ?Sized> : Slice2dIndex<T> {
    fn get_mut(self, slice2d: &mut T) -> Option<&mut Self::Output>;
    unsafe fn get_unchecked_mut(self, slice2d: *mut T) -> *mut Self::Output;
    fn index_mut(self, slice2d: &mut T) -> &mut Self::Output;
}

pub trait Slice2dTrait<T> {
    fn as_ptr(&self) -> *const T;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pitch(&self) -> usize;
}

pub trait MutSlice2dTrait<T>: Slice2dTrait<T> {
    fn as_mut_ptr(&mut self) -> *mut T;
}


pub struct SliceBase2d<'a, T> {
    data: T,
    width: usize,
    pitch: usize,
    height: usize,
    phantom_data: core::marker::PhantomData<&'a T>
}

impl<T> SliceBase2d<'_, T> {
    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub const fn pitch(&self) -> usize {
        self.pitch
    }

    pub const fn buf_len(&self) -> usize {
        self.pitch * self.height
    }

    pub fn index_asserted(&self, (x,y): (usize, usize)) -> usize {
        debug_assert!(x < self.width, "Access out of bounds: width={}, x={}", self.width, x);
        debug_assert!(y < self.height, "Access out of bounds: height={}, y={}", self.height, y);
        x + self.pitch * y
    }
}


pub type Slice2d<'a, T> = SliceBase2d<'a, *const T>;

pub type MutSlice2d<'a, T> = SliceBase2d<'a, *mut T>;

impl<T> Slice2d<'_, T> {

    pub const fn as_ptr(&self) -> *const T {
        self.data
    }

    pub fn buf(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data, self.buf_len()) }
    }

    pub fn get<I: Slice2dIndex<Self>>(&self, index2d: I) -> Option<&I::Output> {
        index2d.get(self)
    } 

    pub unsafe fn get_unchecked<I: Slice2dIndex<Self>>(&self, index2d: I) -> &I::Output {
        unsafe { &*index2d.get_unchecked(self) }
    } 
}

impl<T> MutSlice2d<'_, T> {
    pub const fn as_ptr(&self) -> *const T {
        self.data
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data
    }

    pub fn buf(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data, self.buf_len()) }
    }

    pub fn buf_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.buf_len()) }
    }

    pub fn get<I: MutSlice2dIndex<Self>>(&self, index2d: I) -> Option<&I::Output> {
        index2d.get(self)
    } 

    pub fn get_mut<I: MutSlice2dIndex<Self>>(&mut self, index2d: I) -> Option<&mut I::Output> {
        index2d.get_mut(self)
    } 

    pub unsafe fn get_unchecked<I: MutSlice2dIndex<Self>>(&self, index2d: I) -> &I::Output {
        unsafe { &*index2d.get_unchecked(self) }
    } 

    pub unsafe fn get_unchecked_mut<I: MutSlice2dIndex<Self>>(&mut self, index2d: I) -> &mut I::Output {
        unsafe { &mut *index2d.get_unchecked_mut(self) }
    } 
}


pub unsafe fn from_raw_parts_mut<'a, T>(data: *mut T, width: usize, pitch: usize, height: usize) -> MutSlice2d<'a, T> {
    MutSlice2d { data, width, pitch, height, phantom_data: core::marker::PhantomData{} }
}


pub unsafe fn from_raw_parts<'a, T>(data: *const T, width: usize, pitch: usize, height: usize) -> Slice2d<'a, T> {
    Slice2d { data, width, pitch, height, phantom_data: core::marker::PhantomData{} }
}

impl<'a, T, I> Index<I> for Slice2d<'a, T> where I: Slice2dIndex<Slice2d<'a, T>> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}

impl<'a, T, I> Index<I> for MutSlice2d<'a, T> where I: Slice2dIndex<MutSlice2d<'a, T>> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}


impl<'a, T, I> IndexMut<I> for MutSlice2d<'a, T> where I: MutSlice2dIndex<MutSlice2d<'a, T>> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_mut(self)
    }
}

unsafe impl<'a, T> Slice2dIndex<Slice2d<'a, T>> for (usize, usize) {
    type Output = T;

    fn get(self, slice2d: &Slice2d<'a, T>) -> Option<&'a Self::Output> {
        if self.0 < slice2d.width() && self.1 < slice2d.height() {
            unsafe { Some(&*self.get_unchecked(slice2d)) }
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, slice2d: *const Slice2d<'a, T>) -> *const Self::Output {
        let s: &Slice2d<'a, T> = slice2d.as_ref().unwrap_unchecked();
        let index = s.index_asserted(self);
        debug_assert!(index < s.buf_len(), "Access out of bounds: buffer length={}, index={}", s.buf_len(), index);
        s.as_ptr().add(index)
    }

    fn index<'b>(self, slice2d: &'b Slice2d<'a, T>) -> &'b Self::Output {
        let index = self.0 + (*slice2d).pitch * self.1;
        &(*slice2d).buf()[index]
    }
}
  
unsafe impl<'a, T> Slice2dIndex<MutSlice2d<'a, T>> for (usize, usize) {
    type Output = T;

    fn get(self, slice2d: &MutSlice2d<'a, T>) -> Option<&'a Self::Output> {
        if self.0 < slice2d.width() && self.1 < slice2d.height() {
            unsafe { Some(&*self.get_unchecked(slice2d)) }
        } else {
            None
        }
    }

    unsafe fn get_unchecked(self, slice2d: *const MutSlice2d<'a, T>) -> *const Self::Output {
        let s: &MutSlice2d<'a, T> = slice2d.as_ref().unwrap_unchecked();
        let index = s.index_asserted(self);
        debug_assert!(index < s.buf_len(), "Access out of bounds: buffer length={}, index={}", s.buf_len(), index);
        s.as_ptr().add(index)
    }

    fn index<'b>(self, slice2d: &'b MutSlice2d<'a, T>) -> &'b Self::Output {
        let index = self.0 + (*slice2d).pitch * self.1;
        &(*slice2d).buf()[index]
    }
}


unsafe impl<'a, T> MutSlice2dIndex<MutSlice2d<'a, T>> for (usize, usize) {

    fn get_mut(self, slice2d: &mut MutSlice2d<'a, T>) -> Option<&'a mut Self::Output> {
        if self.0 < slice2d.width() && self.1 < slice2d.height() {
            unsafe { Some(&mut *self.get_unchecked_mut(slice2d)) }
        } else {
            None
        }
    }

    unsafe fn get_unchecked_mut(self, slice2d: *mut MutSlice2d<'a, T>) -> *mut Self::Output {
        let s: &mut MutSlice2d<'a, T> = slice2d.as_mut().unwrap_unchecked();
        let index = s.index_asserted(self);
        debug_assert!(index < s.buf_len(), "Access out of bounds: buffer length={}, index={}", s.buf_len(), index);
        s.as_mut_ptr().add(index)
    }


    fn index_mut<'b>(self, slice2d: &'b mut MutSlice2d<'a, T>) -> &'b mut Self::Output {
        let index = self.0 + (*slice2d).pitch * self.1;
        &mut (*slice2d).buf_mut()[index]
    }
}