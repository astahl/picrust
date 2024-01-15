use core::arch::aarch64::{vtstq_u8, vst1_u8, vget_high_u8};



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

impl<'a> PixelCanvas<'a, u32> {
    pub fn blit8x8(&mut self, src: &[u8; 8], on: u32, off: u32, (x, y): (usize, usize)) -> Result<(), CanvasAccessError> {
        self.check_bounds(x + 8, y + 8)?;
        unsafe {self.blit8x8_unsafe(src, on, off, (x,y));}
        Ok(())
    }

    pub unsafe fn blit8x8_unsafe(&mut self, src: &[u8; 8], on: u32, off: u32, (x, y): (usize, usize)) {
        use core::arch::aarch64::*;
        const MASK: [u8; 16] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];
        let mask = vld1q_u8(MASK.as_ptr());
        let v_off = vdupq_n_u32(off);
        let v_on = vdupq_n_u32(on);

        let start = self.lin(x, y);
        let line0 = self.data.as_mut_ptr().add(start);
        let line1 = line0.add(self.pitch);
        let line2 = line1.add(self.pitch);
        let line3 = line2.add(self.pitch);
        let line4 = line3.add(self.pitch);
        let line5 = line4.add(self.pitch);
        let line6 = line5.add(self.pitch);
        let line7 = line6.add(self.pitch);
        
        let val0 = vld4_dup_u8(src.as_ptr());
        let val1 = vld4_dup_u8(src.as_ptr().add(4));
        let vala = vcombine_u8(val0.0, val0.1);
        let valb = vcombine_u8(val0.2, val0.3);
        let valc = vcombine_u8(val1.0, val1.1);
        let vald = vcombine_u8(val1.2, val1.3);
        let v01 = vtstq_u8(vala, mask);
        let v23 = vtstq_u8(valb, mask);
        let v45 = vtstq_u8(valc, mask);
        let v67 = vtstq_u8(vald, mask);
        
        let expand_u8_to_u32 = |v: uint8x16_t| -> uint32x4x4_t {
            let v0 = vmovl_u8(vget_low_u8(v));
            let v1 = vmovl_u8(vget_high_u8(v));
            let a = vmovl_u16(vget_low_u16(v0)); 
            let b = vmovl_u16(vget_high_u16(v0));
            let c = vmovl_u16(vget_low_u16(v1));
            let d = vmovl_u16(vget_high_u16(v1));
            let e = vtstq_u32(a, a);
            let f = vtstq_u32(b, b);
            let g = vtstq_u32(c, c);
            let h = vtstq_u32(d, d);
            uint32x4x4_t(e, f, g, h)
        };

        let v0 = expand_u8_to_u32(v01);
        let v1 = expand_u8_to_u32(v23);
        let v2 = expand_u8_to_u32(v45);
        let v3 = expand_u8_to_u32(v67);

        let l0 = uint32x4x2_t(vbslq_u32(v0.0, v_on, v_off), vbslq_u32(v0.1, v_on, v_off));
        let l1 = uint32x4x2_t(vbslq_u32(v0.2, v_on, v_off), vbslq_u32(v0.3, v_on, v_off));
        let l2 = uint32x4x2_t(vbslq_u32(v1.0, v_on, v_off), vbslq_u32(v1.1, v_on, v_off));
        let l3 = uint32x4x2_t(vbslq_u32(v1.2, v_on, v_off), vbslq_u32(v1.3, v_on, v_off));
        let l4 = uint32x4x2_t(vbslq_u32(v2.0, v_on, v_off), vbslq_u32(v2.1, v_on, v_off));
        let l5 = uint32x4x2_t(vbslq_u32(v2.2, v_on, v_off), vbslq_u32(v2.3, v_on, v_off));
        let l6 = uint32x4x2_t(vbslq_u32(v3.0, v_on, v_off), vbslq_u32(v3.1, v_on, v_off));
        let l7 = uint32x4x2_t(vbslq_u32(v3.2, v_on, v_off), vbslq_u32(v3.3, v_on, v_off));
        
        vst1q_u32_x2(line0, l0);
        vst1q_u32_x2(line1, l1);
        vst1q_u32_x2(line2, l2);
        vst1q_u32_x2(line3, l3);
        vst1q_u32_x2(line4, l4);
        vst1q_u32_x2(line5, l5);
        vst1q_u32_x2(line6, l6);
        vst1q_u32_x2(line7, l7);
    }
}

impl<'a> PixelCanvas<'a, u8> {
    pub fn blit8x8(&mut self, src: &[u8], on: u8, off: u8, (x, y): (usize, usize)) -> Result<(), CanvasAccessError> {
        self.check_bounds(x + 8, y + 8)?;
        
        unsafe { self.blit8x8_unsafe(src, on, off, (x,y)); }
        Ok(())
    }
    
    pub unsafe fn blit8x8_unsafe(&mut self, src: &[u8], on: u8, off: u8, (x, y): (usize, usize)) {
        use core::arch::aarch64::*;
        const MASK: [u8; 16] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];
        let start = self.lin(x, y);
        let line0 = self.data.as_mut_ptr().add(start);
        let line1 = line0.add(self.pitch);
        let line2 = line1.add(self.pitch);
        let line3 = line2.add(self.pitch);
        let line4 = line3.add(self.pitch);
        let line5 = line4.add(self.pitch);
        let line6 = line5.add(self.pitch);
        let line7 = line6.add(self.pitch);
        
        let mask = vld1q_dup_u8(MASK.as_ptr());
        let val0 = vld4_dup_u8(src.as_ptr());
        let val1 = vld4_dup_u8(src.as_ptr().add(4));
        let vala = vcombine_u8(val0.0, val0.1);
        let valb = vcombine_u8(val0.2, val0.3);
        let valc = vcombine_u8(val1.0, val1.1);
        let vald = vcombine_u8(val1.2, val1.3);
        let v0 = vtstq_u8(vala, mask);
        let v1 = vtstq_u8(valb, mask);
        let v2 = vtstq_u8(valc, mask);
        let v3 = vtstq_u8(vald, mask);
        
        vst1_u8(line0, vget_high_u8(v0));
        vst1_u8(line1, vget_low_u8(v0));
        vst1_u8(line2, vget_high_u8(v1));
        vst1_u8(line3, vget_low_u8(v1));
        vst1_u8(line4, vget_high_u8(v2));
        vst1_u8(line5, vget_low_u8(v2));
        vst1_u8(line6, vget_high_u8(v3));
        vst1_u8(line7, vget_low_u8(v3));
    }
}
