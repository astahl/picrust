pub struct PixelCanvas<'a, T> {
    width: usize,
    height: usize,
    pitch: usize,
    data: &'a mut [T]
}

pub type PixelCanvasU8<'a> = PixelCanvas<'a, u8>;

#[derive(Debug)]
pub enum CanvasAccessError {
    OverflowX,
    OverflowY,
    UnsortedCoordinates,
}

impl<'a, T> PixelCanvas<'a, T> {
    pub fn with_slice(width: usize, height: usize, pitch: usize, slice: &'a mut [T]) -> Option<Self> {
        let size = Self::required_size(width, pitch);
        if size <= slice.len() {
            unsafe { Some(Self::with_slice_unchecked(width, height, pitch, slice)) }
        } else {
            None 
        }
    }

    pub unsafe fn with_slice_unchecked(width: usize, height: usize, pitch: usize, slice: &'a mut [T]) -> Self {
        let size = Self::required_size(width, pitch);
        Self{ width, height, pitch, data: slice.get_unchecked_mut(0..size) }
    }


    pub unsafe fn from_raw_parts(width: usize, height: usize, pitch: usize, ptr: *mut T) -> Self {
        Self{ width, height, pitch, data: core::slice::from_raw_parts_mut(ptr, Self::required_size(width, pitch)) }
    }

    pub fn put(&mut self, value: T, (x, y): (usize, usize)) -> Result<(), CanvasAccessError> {
        self.check_bounds(x, y)?;
        unsafe {
            self.put_unchecked(value, (x, y));
        }
        Ok(())
    }

    pub unsafe fn put_unchecked(&mut self, value: T, (x, y): (usize, usize)) {
        *self.data.get_unchecked_mut(x + y * self.pitch) = value;
    }

    const fn required_size(width: usize, pitch: usize) -> usize {
        width * pitch
    }

    const fn lin(&self, x: usize, y: usize) -> usize {
        self.pitch * y + x
    }

    const fn check_bounds(&self, x: usize, y: usize) -> Result<(), CanvasAccessError> {
        if x >= self.width {
            Err(CanvasAccessError::OverflowX)
        } else if y >= self.height {
            Err(CanvasAccessError::OverflowY)
        } else {
            Ok(())
        }
    }
}

impl<'a, T> PixelCanvas<'a, T> where T: Clone + Copy {
    
    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }

    pub fn fill_lines<I: core::ops::RangeBounds<usize>>(&mut self, value: T, range: I) -> Result<(), CanvasAccessError>{
        let mut start_line: usize = match range.start_bound() {
            core::ops::Bound::Included(y) => *y,
            core::ops::Bound::Excluded(y) => y + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let mut end_line: usize = match range.end_bound() {
            core::ops::Bound::Included(y) => y + 1,
            core::ops::Bound::Excluded(y) => *y,
            core::ops::Bound::Unbounded => self.height,
        };

        if end_line < start_line {
            return Err(CanvasAccessError::UnsortedCoordinates);
        }

        self.check_bounds(0, start_line).and(self.check_bounds(0, end_line - 1))?;

        unsafe {
            let lines = end_line - start_line;
            let from_ptr = self.data.as_mut_ptr().add(start_line * self.pitch);
            core::slice::from_raw_parts_mut(from_ptr, lines * self.pitch).fill(value);
        }
        Ok(())
    }

    pub fn fill_rect(&mut self, value: T, (x0, y0): (usize, usize), (x1, y1): (usize, usize)) -> Result<(), CanvasAccessError>  {
        self.check_bounds(x0, y0).and(self.check_bounds(x1 - 1, y1 - 1))?;
        unsafe { self.fill_rect_unchecked(value, (x0, y0), (x1, y1)); }
        Ok(())
    }

    pub unsafe fn fill_rect_unchecked(&mut self, value: T, (x0, mut y0): (usize, usize), (x1, y1): (usize, usize)) {
        let len = x1 - x0;
        let mut from_ptr = self.data.as_mut_ptr().add(self.lin(x0, y0));
        loop {
            core::slice::from_raw_parts_mut(from_ptr, len).fill(value);
            from_ptr = from_ptr.add(self.pitch);
            y0 += 1;
            if y0 == y1 {
                break;
            }
        }
    }
    
    pub fn fill_bytes(&mut self, value: u8) {
        unsafe {
            core::ptr::write_bytes(self.data.as_mut_ptr(), value, self.pitch * self.height);
        }
    }
}

