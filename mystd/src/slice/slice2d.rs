use core::{ops::{Index, IndexMut, RangeBounds}, usize};

pub mod iter;

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

pub trait Slice2dTrait {
    type Element;

    fn as_ptr(&self) -> *const Self::Element;
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn pitch(&self) -> usize;
    
    fn is_empty(&self) -> bool {
        self.width() == 0 || self.height() == 0
    }

    fn buf_len(&self) -> usize {
        self.pitch() * self.width()
    }

    fn buf_slice(&self) -> &[Self::Element] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.buf_len())}
    }

    fn index_asserted(&self, (x,y): (usize, usize)) -> usize {
        debug_assert!(x < self.width(), "Access out of bounds: width={}, x={}", self.width(), x);
        debug_assert!(y < self.height(), "Access out of bounds: height={}, y={}", self.height(), y);
        x + self.pitch() * y
    }

    fn rows(&self) -> iter::RowIter<Self::Element> {
        iter::RowIter::new(self.as_ptr(), self.width(), self.pitch(), self.height())
    }

    fn row_unchecked(&self, row_idx: usize) -> &[Self::Element] {
        unsafe { core::slice::from_raw_parts(self.as_ptr().wrapping_add(row_idx * self.pitch()), self.width())}
    }

    fn col(&self, col_idx: usize) -> iter::ColIter<Self::Element> {
        if col_idx < self.width() {
            iter::ColIter::new(self.as_ptr().wrapping_add(col_idx), self.pitch(), self.height())
        } else {
            iter::ColIter::new(self.as_ptr(), 0, 0)
        }
    }

    fn as_slice2d(&self) -> Slice2d<Self::Element> {
        unsafe { Slice2d::from_raw_parts(self.as_ptr(), self.width(), self.pitch(), self.height())}
    }
}

pub trait MutSlice2dTrait: Slice2dTrait {
    fn as_mut_ptr(&mut self) -> *mut Self::Element;

    fn buf_mut_slice(&mut self) -> &mut [Self::Element] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.buf_len())}
    }

    fn rows_mut(&mut self) -> iter::RowIterMut<Self::Element> {
        iter::RowIterMut::new(self.as_mut_ptr(), self.width(), self.pitch(), self.height())
    }

    fn row_mut_unchecked(&mut self, row_idx: usize) -> &mut [Self::Element] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr().wrapping_add(row_idx * self.pitch()), self.width())}
    }

    fn col_mut(&mut self, col_idx: usize) -> iter::ColIterMut<Self::Element> {
        if col_idx < self.width() {
            iter::ColIterMut::new(self.as_mut_ptr().wrapping_add(col_idx), self.pitch(), self.height())
        } else {
            iter::ColIterMut::new(self.as_mut_ptr(), 0, 0)
        }
    }

    fn as_mut_slice2d(&mut self) -> MutSlice2d<Self::Element> {
        unsafe { MutSlice2d::from_raw_parts(self.as_mut_ptr(), self.width(), self.pitch(), self.height())}
    }
}


pub struct Slice2d<'a, T> {
    data: *const T,
    width: usize,
    pitch: usize,
    height: usize,
    phantom_data: core::marker::PhantomData<&'a T>
}

pub struct MutSlice2d<'a, T> {
    data: *mut T,
    width: usize,
    pitch: usize,
    height: usize,
    phantom_data: core::marker::PhantomData<&'a T>
}

impl<T> Slice2dTrait for Slice2d<'_, T> {
    type Element = T;
    
    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    #[inline]
    fn pitch(&self) -> usize {
        self.pitch
    }
    
    #[inline]
    fn as_ptr(&self) -> *const Self::Element {
        self.data
    }
}

fn range_to_offset_len<R: core::ops::RangeBounds<usize>>(range: R, mut bound: (usize, usize)) -> (usize, usize) {
    match range.start_bound() {
        core::ops::Bound::Included(first) => bound.0 = *first,
        core::ops::Bound::Excluded(start) => bound.0 = *start + 1,
        core::ops::Bound::Unbounded => {},
    }

    match range.end_bound() {
        core::ops::Bound::Included(last) => bound.1 = *last + 1,
        core::ops::Bound::Excluded(end) => bound.1 = *end,
        core::ops::Bound::Unbounded => {},
    }

    if bound.0 > bound.1 {
        (bound.0, 0)
    } else {
        bound.1 -= bound.0;
        bound
    }

}

impl<T> Slice2d<'_, T> {
    pub fn with_slice(buf: &[T], width: usize, pitch: usize, height: usize) -> Option<(Self, &[T])> {
        assert!(width <= pitch, "Width must not be larger than pitch.");
        let required_len = pitch * height;
        if required_len > buf.len() {
            None
        } else {
            Some((unsafe { Self::from_raw_parts(buf.as_ptr(), width, pitch, height) }, &buf[required_len..]))
        }
    }

    pub unsafe fn from_raw_parts(data: *const T, width: usize, pitch: usize, height: usize) -> Self{
        Self { data, width, pitch, height, phantom_data: core::marker::PhantomData }
    }

    pub fn get<I: Slice2dIndex<Self>>(&self, index2d: I) -> Option<&I::Output> {
        index2d.get(self)
    } 

    pub unsafe fn get_unchecked<I: Slice2dIndex<Self>>(&self, index2d: I) -> &I::Output {
        unsafe { &*index2d.get_unchecked(self) }
    } 

    pub fn split_at_line(&self, line_n: usize) -> (Self, Self) {
        assert!(line_n <= self.height);
        unsafe {(
            Self::from_raw_parts(self.data, self.width, self.pitch, line_n),
            Self::from_raw_parts(self.data.wrapping_add(line_n * self.pitch), self.width, self.pitch, self.height - line_n)
        )}
    }

    pub fn split_at_col(&self, col_n: usize) -> (Self, Self) {
        assert!(col_n <= self.width);
        unsafe {(
            Self::from_raw_parts(self.data, col_n, self.pitch, self.height),
            Self::from_raw_parts(self.data.wrapping_add(col_n), self.width - col_n, self.pitch, self.height)
        )}
    }

    pub fn sub_slice2d<R: core::ops::RangeBounds<usize>, S: core::ops::RangeBounds<usize>>(&self, (col_range, line_range): (R, S)) -> Slice2d<T> {
        let (x, width) = range_to_offset_len(col_range, (0, self.width));
        let (y, height) = range_to_offset_len(line_range, (0, self.width));
        unsafe {
            let data = self.get_unchecked((x,y)) as *const T;
            Slice2d::from_raw_parts(data, width, self.pitch, height)
        }
    }
}

impl<T> MutSlice2d<'_, T> {
    pub fn with_mut_slice(buf: &mut [T], width: usize, pitch: usize, height: usize) -> Option<(Self, &mut [T])> {
        assert!(width <= pitch, "Width must not be larger than pitch.");
        let required_len = pitch * height;
        if required_len > buf.len() {
            None
        } else {
            Some((unsafe { Self::from_raw_parts(buf.as_mut_ptr(), width, pitch, height) }, &mut buf[required_len..]))
        }
    }

    pub unsafe fn from_raw_parts(data: *mut T, width: usize, pitch: usize, height: usize) -> Self{
        Self { data, width, pitch, height, phantom_data: core::marker::PhantomData }
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

    pub fn split_at_line(&self, line_n: usize) -> (Slice2d<T>, Slice2d<T>) {
        assert!(line_n <= self.height);
        unsafe {(
            Slice2d::from_raw_parts(self.data, self.width, self.pitch, line_n),
            Slice2d::from_raw_parts(self.data.wrapping_add(line_n * self.pitch), self.width, self.pitch, self.height - line_n)
        )}
    }

    pub fn split_at_line_mut(&self, line_n: usize) -> (Self, Self) {
        assert!(line_n <= self.height);
        unsafe {(
            Self::from_raw_parts(self.data, self.width, self.pitch, line_n),
            Self::from_raw_parts(self.data.wrapping_add(line_n * self.pitch), self.width, self.pitch, self.height - line_n)
        )}
    }

    pub fn split_at_col(&self, col_n: usize) -> (Slice2d<T>, Slice2d<T>) {
        assert!(col_n <= self.width);
        unsafe {(
            Slice2d::from_raw_parts(self.data, col_n, self.pitch, self.height),
            Slice2d::from_raw_parts(self.data.wrapping_add(col_n), self.width - col_n, self.pitch, self.height)
        )}
    }

    pub fn split_at_col_mut(&self, col_n: usize) -> (Self, Self) {
        assert!(col_n <= self.width);
        unsafe {(
            Self::from_raw_parts(self.data, col_n, self.pitch, self.height),
            Self::from_raw_parts(self.data.wrapping_add(col_n), self.width - col_n, self.pitch, self.height)
        )}
    }
}

impl<T> Slice2dTrait for MutSlice2d<'_, T> {
    type Element = T;

    #[inline]
    fn as_ptr(&self) -> *const Self::Element {
        self.data
    }

    #[inline]
    fn width(&self) -> usize {
        self.width
    }

    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    #[inline]
    fn pitch(&self) -> usize {
        self.pitch
    }
}

impl<T> MutSlice2dTrait for MutSlice2d<'_, T> {
    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Element {
        self.data
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
        &(*slice2d).buf_slice()[index]
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
        &(*slice2d).buf_slice()[index]
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
        &mut (*slice2d).buf_mut_slice()[index]
    }
}


impl<T: PartialEq, R: Slice2dTrait<Element=T>> core::cmp::PartialEq<R> for Slice2d<'_, T> {
    fn eq(&self, other: &R) -> bool {
        self.rows().zip(other.rows()).all(|(l,r)| l.eq(r))
    }
}

impl<T: PartialEq, R: Slice2dTrait<Element=T>> core::cmp::PartialEq<R> for MutSlice2d<'_, T> {
    fn eq(&self, other: &R) -> bool {
        self.rows().zip(other.rows()).all(|(l,r)| l.eq(r))
    }
}


impl<T: core::fmt::Debug> core::fmt::Debug for Slice2d<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            let mut w = &mut f.debug_list(); 
            for r in self.rows() {
                w = w.entry(&format_args!("{:?}", r));
            }
            w.finish()
        } else {
            f.debug_list().entries(self.rows()).finish()
        }
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for MutSlice2d<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            let mut w = &mut f.debug_list(); 
            for r in self.rows() {
                w = w.entry(&format_args!("{:?}", r));
            }
            w.finish()
        } else {
            f.debug_list().entries(self.rows()).finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_slice() {
        let buf = [0_u8; 32];
        let (a, b) = Slice2d::with_slice(buf.as_slice(), 5, 6, 5).expect("Should work");
        assert_eq!(2, b.len());
        assert_eq!(5, a.height());
        assert_eq!(6, a.pitch());
        assert_eq!(5, a.width());
    }

    #[test]
    fn with_slice_mut() {
        let mut buf = [0_u8; 32];
        let (a, b) = MutSlice2d::with_mut_slice(buf.as_mut_slice(), 5, 6, 5).expect("Should work");
        assert_eq!(2, b.len());
        assert_eq!(5, a.height());
        assert_eq!(6, a.pitch());
        assert_eq!(5, a.width());
    }

    #[test]
    fn sub_slice2d() {
        let buf = core::array::from_fn::<usize, 16, _>(|i| i);
        let (a, _) = Slice2d::with_slice(&buf, 4, 4, 4).expect("Should work");
        assert_eq!(a, crate::arr2d!(
            [0, 1, 2, 3], 
            [4, 5, 6, 7], 
            [8, 9, 10,11], 
            [12,13,14,15]));
        assert_eq!(a.sub_slice2d((1.., 1..)), crate::arr2d!(
            [5, 6, 7], 
            [9,10,11], 
            [13,14,15]));
        
        assert_eq!(a.sub_slice2d((1..3, 1..3)), crate::arr2d!(
            [5, 6], 
            [9,10]));

        assert_eq!(a.sub_slice2d((1..=1, 1..2)), crate::arr2d!([5]));
        assert_eq!(a.sub_slice2d((.., 3..=3)), crate::arr2d!([12,13,14,15]));
    }
}