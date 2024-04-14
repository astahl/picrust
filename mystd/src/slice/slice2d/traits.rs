use super::Slice2d;
use super::MutSlice2d;
use super::iter;

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

pub trait Point2d {
    type Coord;
    fn x(&self) -> &Self::Coord;
    fn y(&self) -> &Self::Coord;
    fn to_xy(self) -> (Self::Coord, Self::Coord);
}

impl<T> Point2d for (T, T) {
    type Coord = T;
    fn x(&self) -> &T {
        &self.0
    }

    fn y(&self) -> &T {
        &self.1
    }
    
    fn to_xy(self) -> (Self::Coord, Self::Coord) {
        self
    }
}

impl<T> Point2d for &(T, T) where T: Copy {
    type Coord = T;
    fn x(&self) -> &T {
        &self.0
    }

    fn y(&self) -> &T {
        &self.1
    }
    
    fn to_xy(self) -> (Self::Coord, Self::Coord) {
        *self
    }
}

pub trait RangeBounds2d<V>  {
    type HorizontalBounds: core::ops::RangeBounds<V>;
    type VerticalBounds: core::ops::RangeBounds<V>;
    fn horizontal_bounds(&self) -> &Self::HorizontalBounds;
    fn vertical_bounds(&self) -> &Self::VerticalBounds;

    fn contains<P: Point2d<Coord = V>>(&self, point: &P) -> bool where V: PartialOrd {
        use core::ops::RangeBounds;
        self.horizontal_bounds().contains(point.x()) && self.vertical_bounds().contains(point.y())
    }
}

impl<T, U, V> RangeBounds2d<V> for (T, U) where T: core::ops::RangeBounds<V>, U: core::ops::RangeBounds<V> {
    fn horizontal_bounds(&self) -> &T {
        &self.0
    }

    fn vertical_bounds(&self) -> &U {
        &self.1
    }
    
    type HorizontalBounds = T;
    
    type VerticalBounds = U;
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

    fn is_continuous(&self) -> bool {
        self.pitch() == self.width()
    }

    fn stride(&self) -> usize {
        self.pitch() - self.width()
    }

    fn buf_len(&self) -> usize {
        self.pitch() * self.height()
    }

    fn buf_slice(&self) -> &[Self::Element] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.buf_len())}
    }

    fn index_asserted<P: Point2d<Coord = usize>>(&self, p: P) -> usize {
        let (x,y) = p.to_xy();
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

    fn sub_slice2d<R>(&self, range2d: R) -> Slice2d<Self::Element> 
    where 
        R: RangeBounds2d<usize>, {
        let (x, width) = range_to_offset_len(range2d.horizontal_bounds(), (0, self.width()));
        let (y, height) = range_to_offset_len(range2d.vertical_bounds(), (0, self.height()));
        unsafe {
            let ptr = self.as_ptr().wrapping_add(self.index_asserted((x,y)));
            Slice2d::from_raw_parts(ptr, width, self.pitch(), height)
        }
    }

    fn enumerate(&self) -> iter::Enumerate2d<Self::Element> {
        iter::Enumerate2d::new(self.as_ptr(), self.width(), self.pitch(), self.height())
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

    fn sub_mut_slice2d<R>(&mut self, range2d: R) -> MutSlice2d<Self::Element> 
    where 
        R: RangeBounds2d<usize>, {
        let (x, width) = range_to_offset_len(range2d.horizontal_bounds(), (0, self.width()));
        let (y, height) = range_to_offset_len(range2d.vertical_bounds(), (0, self.height()));
        unsafe {
            let ptr = self.as_mut_ptr().wrapping_add(self.index_asserted((x,y)));
            MutSlice2d::from_raw_parts(ptr, width, self.pitch(), height)
        }
    }

    fn fill(&mut self, value: Self::Element) where Self::Element: Copy {
        if self.width() == 1 && self.height() == 1 {
            unsafe {
                *self.as_mut_ptr() = value; 
            }
        } else if self.width() == self.pitch() {
            self.buf_mut_slice().fill(value);
        } else {
            for dst in self.rows_mut() {
                dst.fill(value)
            }
        }
    }

    fn copy_from_slice2d<S: Slice2dTrait<Element = Self::Element>>(&mut self, other: &S) where Self::Element: Copy {
        assert_eq!(self.width(), other.width());
        assert_eq!(self.height(), other.height());
        if self.width() == 1 && self.height() == 1 {
            unsafe {
                *self.as_mut_ptr() = *other.as_ptr(); 
            }
        } else if self.is_continuous() && other.is_continuous() {
            self.buf_mut_slice().copy_from_slice(other.buf_slice());
        } else {
            for (dst, src) in self.rows_mut().zip(other.rows()) {
                dst.copy_from_slice(src)
            }
        }
    }

    unsafe fn copy_buf_unchecked(&mut self, other: *const Self::Element) {
        core::ptr::copy_nonoverlapping(other, self.as_mut_ptr(), self.buf_len())
    }

    fn enumerate_mut(&mut self) -> iter::Enumerate2dMut<Self::Element> {
        iter::Enumerate2dMut::new(self.as_mut_ptr(), self.width(), self.pitch(), self.height())
    }
}

pub trait DebugSlice2dTrait : Slice2dTrait + core::fmt::Debug
where 
    Self::Element : core::fmt::Debug, 
    Self: Sized 
{
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result  {
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

pub trait PartialEqSlice2dTrait<R: Slice2dTrait> : Slice2dTrait + core::cmp::PartialEq<R>
    where Self::Element: core::cmp::PartialEq<R::Element>
{
    fn cmp_eq(&self, other: &R) -> bool {
        self.width() == other.width() &&
        self.height() == other.height() &&
        self.rows().zip(other.rows()).all(|(l,r)| l == r)
    }
}

pub fn range_to_offset_len<R: core::ops::RangeBounds<usize>>(range: &R, mut bound: (usize, usize)) -> (usize, usize) {
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