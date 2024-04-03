use core::usize;
use crate::slice::slice2d::{traits::{MutSlice2dTrait, Slice2dTrait}, MutSlice2d};

pub struct PixelCanvas<'a, T> {
    pub(crate) data: &'a mut MutSlice2d<'a, T>,
}

impl<'a, T> AsRef<MutSlice2d<'a, T>> for PixelCanvas<'a, T> {
    fn as_ref(&self) -> &MutSlice2d<'a, T> {
        &self.data
    }
}

impl<'a, T> AsMut<MutSlice2d<'a, T>> for PixelCanvas<'a, T> {
    fn as_mut(&mut self) -> &mut MutSlice2d<'a, T> {
        self.data
    }
}

pub type PixelCanvasU8<'a> = PixelCanvas<'a, u8>;

#[derive(Debug)]
pub enum CanvasAccessError {
    OverflowX,
    OverflowY,
    PitchMismatch,
    UnsortedCoordinates,
    OverlappingMemoryRegions,
}

pub enum BoundsStrategy {
    Panic,
    Fail,
    Clip,
    Mirror,
    Repeat,
}

impl BoundsStrategy {
    pub const fn test_signed(&self, upper_limit: usize, value: isize) -> Option<usize> {
        let unsigned_value = value as usize;
        if unsigned_value < upper_limit {
            Some(value as usize)
        } else {
            let absolute_value = value.unsigned_abs();
            match self {
                BoundsStrategy::Panic => panic!(),
                BoundsStrategy::Fail => None,
                BoundsStrategy::Clip => {
                    if value.is_positive() {
                        Some(upper_limit - 1)
                    } else {
                        Some(0)
                    }
                },
                BoundsStrategy::Mirror => {
                    let remainder = absolute_value % upper_limit;
                    if value.is_positive() {
                        Some(upper_limit.wrapping_sub(remainder))
                    } else {
                        Some(remainder)
                    }
                }
                BoundsStrategy::Repeat => {
                    let remainder = absolute_value % upper_limit;
                    if value.is_negative() {
                        Some(upper_limit.wrapping_sub(remainder))
                    } else {
                        Some(remainder)
                    }
                }
            }
        }
    }

    pub const fn test_unsigned(&self, upper_limit: usize, value: usize) -> Option<usize> {
        if value < upper_limit {
            Some(value as usize)
        } else {
            match self {
                BoundsStrategy::Panic => panic!(),
                BoundsStrategy::Fail => None,
                BoundsStrategy::Clip => {
                    Some(upper_limit - 1)
                },
                BoundsStrategy::Mirror => {
                    let remainder = value % upper_limit;
                    Some(upper_limit - remainder)
                }
                BoundsStrategy::Repeat => {
                    Some(value % upper_limit)
                }
            }
        }
    }
}

impl<'a, T> PixelCanvas<'a, T> {
    pub fn with_slice2d(
        slice: &'a mut MutSlice2d<'a, T>,
    ) -> Self {
        Self { data: slice }
    }

    pub fn copy_from(&mut self, other: &Self) where T: Copy {
        self.data.copy_from_slice2d(other.data)
    }

    pub fn put(&mut self, value: T, (x, y): (usize, usize)) -> Result<(), CanvasAccessError> {
        self.check_bounds(x, y)?;
        unsafe {
            self.put_unchecked(value, (x, y));
        }
        Ok(())
    }

    pub unsafe fn put_unchecked(&mut self, value: T, (x, y): (usize, usize)) {
        *self.data.get_unchecked_mut((x,y)) = value;
    }

    pub(crate) fn check_bounds(&self, x: usize, y: usize) -> Result<(), CanvasAccessError> {
        if x >= self.data.width() {
            Err(CanvasAccessError::OverflowX)
        } else if y >= self.data.height() {
            Err(CanvasAccessError::OverflowY)
        } else {
            Ok(())
        }
    }

    pub(crate) fn lin(&self, x: usize, y: usize) -> usize {
        x + y * self.data.pitch()
    }

    pub(crate) fn bounded_coords(&self, (x, y): (isize, isize), horizontal_bounds: BoundsStrategy, vertical_bounds: BoundsStrategy) -> Option<(usize, usize)> {
        Some((horizontal_bounds.test_signed(self.data.width(), x)?, vertical_bounds.test_signed(self.data.height(), y)?))
    }
}

impl<'a, T> PixelCanvas<'a, T>
where
    T: Clone + Copy,
{
    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }

    pub fn fill_lines<I: core::ops::RangeBounds<usize>>(
        &mut self,
        value: T,
        range: I,
    ) -> Result<(), CanvasAccessError> {
        let start_line: usize = match range.start_bound() {
            core::ops::Bound::Included(y) => *y,
            core::ops::Bound::Excluded(y) => y + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end_line: usize = match range.end_bound() {
            core::ops::Bound::Included(y) => y + 1,
            core::ops::Bound::Excluded(y) => *y,
            core::ops::Bound::Unbounded => self.data.height(),
        };

        if end_line < start_line {
            return Err(CanvasAccessError::UnsortedCoordinates);
        }

        self.check_bounds(0, start_line)
            .and(self.check_bounds(0, end_line - 1))?;

        unsafe {
            let lines = end_line - start_line;
            let from_ptr = self.data.as_mut_ptr().add(start_line * self.data.pitch());
            core::slice::from_raw_parts_mut(from_ptr, lines * self.data.pitch()).fill(value);
        }
        Ok(())
    }

    pub fn fill_rect(
        &mut self,
        value: T,
        (x0, y0): (usize, usize),
        (x1, y1): (usize, usize),
    ) -> Result<(), CanvasAccessError> {
        self.check_bounds(x0, y0)
            .and(self.check_bounds(x1 - 1, y1 - 1))?;
        unsafe {
            self.fill_rect_unchecked(value, (x0, y0), (x1, y1));
        }
        Ok(())
    }

    pub unsafe fn fill_rect_unchecked(
        &mut self,
        value: T,
        (x0, mut y0): (usize, usize),
        (x1, y1): (usize, usize),
    ) {
        let len = x1 - x0;
        let mut from_ptr = self.data.as_mut_ptr().add(self.lin(x0, y0));
        loop {
            core::slice::from_raw_parts_mut(from_ptr, len).fill(value);
            from_ptr = from_ptr.add(self.data.pitch());
            y0 += 1;
            if y0 == y1 {
                break;
            }
        }
    }

    pub fn fill_bytes(&mut self, value: u8) {
        unsafe {
            core::ptr::write_bytes(self.data.as_mut_ptr(), value, self.data.pitch() * self.data.height());
        }
    }

    pub fn scale_in_place(&mut self, x_repeat: usize, y_repeat: usize) {
        let rows = self.data.height() / y_repeat;
        let cols = self.data.width() / x_repeat;
        unsafe {
            // stretch lines (starting at the top right)
            if x_repeat > 1 {
                for y in 0..rows {
                    let line_offset = y * self.data.pitch();
                    let mut dst_ptr = self.data.as_mut_ptr().add(cols * x_repeat + line_offset);
                    let mut src_ptr = self.data.as_ptr().add(cols + line_offset);
                    while src_ptr != dst_ptr {
                        let value = *src_ptr;
                        for _ in 0..x_repeat {
                            *dst_ptr = value;
                            dst_ptr = dst_ptr.offset(-1);
                        }
                        src_ptr = src_ptr.offset(-1);
                    }
                }
            }
            // repeat lines (starting at the bottom left)
            if y_repeat > 1 {
                let line_offset = rows * self.data.pitch();
                let row_step = -(self.data.pitch() as isize);
                let mut dst = self.data.as_mut_ptr().add(y_repeat * line_offset);
                let mut src = self.data.as_ptr().add(line_offset);
                let mut repeat_counter = 0;
                while src != dst {
                    if repeat_counter == 0 {
                        src = src.offset(row_step);
                        repeat_counter = y_repeat;
                    }
                    core::ptr::copy_nonoverlapping(src, dst, self.data.width());
                    dst = dst.offset(row_step);
                    repeat_counter -= 1;
                }
            }
        }
    }
}

impl<'a> PixelCanvas<'a, u32> {
    pub fn blit8x8(
        &mut self,
        src: &[u8; 8],
        on: u32,
        off: u32,
        (x, y): (usize, usize),
    ) -> Result<(), CanvasAccessError> {
        self.check_bounds(x + 8, y + 8)?;
        unsafe {
            self.blit8x8_unsafe(src, on, off, (x, y));
        }
        Ok(())
    }

    pub unsafe fn blit8x8_unsafe(
        &mut self,
        src: &[u8; 8],
        on: u32,
        off: u32,
        (x, y): (usize, usize),
    ) {
        use core::arch::aarch64::*;
        pub(crate) const MASK: [u8; 16] = [
            0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04,
            0x02, 0x01,
        ];
        let mask = vld1q_u8(MASK.as_ptr());
        let v_off = vdupq_n_u32(off);
        let v_on = vdupq_n_u32(on);

        let line_step = self.data.pitch();

        let start = self.lin(x, y);
        let line0 = self.data.as_mut_ptr().add(start);
        let line1 = line0.add(line_step);
        let line2 = line1.add(line_step);
        let line3 = line2.add(line_step);
        let line4 = line3.add(line_step);
        let line5 = line4.add(line_step);
        let line6 = line5.add(line_step);
        let line7 = line6.add(line_step);

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

    pub fn blit8x8_line(
        &mut self,
        src: &[u64],
        on: u32,
        off: u32,
        (x, y): (usize, usize),
    ) -> Result<(), CanvasAccessError> {
        self.check_bounds(x + 8 * src.len() - 1, y + 7)?;
        unsafe {
            self.blit8x8_line_unsafe(src, on, off, (x, y));
        }
        Ok(())
    }

    pub unsafe fn blit8x8_line_unsafe(
        &mut self,
        src: &[u64],
        on: u32,
        off: u32,
        (x, y): (usize, usize),
    ) {
        use core::arch::aarch64::*;
        pub(crate) const MASK: [u8; 16] = [
            0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04,
            0x02, 0x01,
        ];
        let mask = vld1q_u8(MASK.as_ptr());
        let v_off = vdupq_n_u32(off);
        let v_on = vdupq_n_u32(on);

        let line_step = self.data.pitch();

        let start = self.lin(x, y);
        let mut line0 = self.data.as_mut_ptr().add(start);
        let mut line1 = line0.add(line_step);
        let mut line2 = line1.add(line_step);
        let mut line3 = line2.add(line_step);
        let mut line4 = line3.add(line_step);
        let mut line5 = line4.add(line_step);
        let mut line6 = line5.add(line_step);
        let mut line7 = line6.add(line_step);

        for tile in src {
            let bytes = tile.to_le_bytes();
            let val0 = vld4_dup_u8(bytes.as_ptr());
            let val1 = vld4_dup_u8(bytes.as_ptr().add(4));
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

            line0 = line0.add(8);
            line1 = line1.add(8);
            line2 = line2.add(8);
            line3 = line3.add(8);
            line4 = line4.add(8);
            line5 = line5.add(8);
            line6 = line6.add(8);
            line7 = line7.add(8);
        }
    }
}

impl<'a> PixelCanvas<'a, u8> {
    pub fn blit8x8(
        &mut self,
        src: &[u8],
        on: u8,
        off: u8,
        (x, y): (usize, usize),
    ) -> Result<(), CanvasAccessError> {
        self.check_bounds(x + 8, y + 8)?;

        unsafe {
            self.blit8x8_unsafe(src, on, off, (x, y));
        }
        Ok(())
    }

    pub unsafe fn blit8x8_unsafe(&mut self, src: &[u8], on: u8, off: u8, (x, y): (usize, usize)) {
        use core::arch::aarch64::*;
        pub(crate) const MASK: [u8; 16] = [
            0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04,
            0x02, 0x01,
        ];
        let v_off = vdupq_n_u8(off);
        let v_on = vdupq_n_u8(on);
        let start = self.lin(x, y);
        let line0 = self.data.as_mut_ptr().add(start);
        let line1 = line0.add(self.data.pitch());
        let line2 = line1.add(self.data.pitch());
        let line3 = line2.add(self.data.pitch());
        let line4 = line3.add(self.data.pitch());
        let line5 = line4.add(self.data.pitch());
        let line6 = line5.add(self.data.pitch());
        let line7 = line6.add(self.data.pitch());

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
        let v0 = vbslq_u8(v0, v_on, v_off);
        let v1 = vbslq_u8(v1, v_on, v_off);
        let v2 = vbslq_u8(v2, v_on, v_off);
        let v3 = vbslq_u8(v3, v_on, v_off);

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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn bounds_strategy_signed_repeat_works() {
        let strategy = BoundsStrategy::Repeat;
        assert_eq!(Some(1), strategy.test_signed(13, 1));
        assert_eq!(Some(1), strategy.test_signed(13, 14));
        assert_eq!(Some(1), strategy.test_signed(13, 27));
        assert_eq!(Some(1), strategy.test_signed(13, -12));
        assert_eq!(Some(1), strategy.test_signed(13, -25));
    }

    #[test]
    fn bounds_strategy_unsigned_repeat_works() {
        let strategy = BoundsStrategy::Repeat;
        assert_eq!(Some(1), strategy.test_unsigned(13, 1));
        assert_eq!(Some(1), strategy.test_unsigned(13, 14));
        assert_eq!(Some(1), strategy.test_unsigned(13, 27));
    }

    #[test]
    fn bounds_strategy_signed_mirror_works() {
        let strategy = BoundsStrategy::Mirror;
        assert_eq!(Some(1), strategy.test_signed(13, 1));
        assert_eq!(Some(13), strategy.test_signed(13, 13));
        assert_eq!(Some(1), strategy.test_signed(13, 38));
        assert_eq!(Some(12), strategy.test_signed(13, -12));
        assert_eq!(Some(12), strategy.test_signed(13, -25));
    }

    #[test]
    fn bounds_strategy_unsigned_mirror_works() {
        let strategy = BoundsStrategy::Mirror;
        assert_eq!(Some(1), strategy.test_unsigned(13, 1));
        assert_eq!(Some(13), strategy.test_unsigned(13, 13));
        assert_eq!(Some(1), strategy.test_unsigned(13, 38));
    }

    #[test]
    fn bounds_strategy_signed_clip_works() {
        let strategy = BoundsStrategy::Clip;
        assert_eq!(Some(1), strategy.test_signed(13, 1));
        assert_eq!(Some(12), strategy.test_signed(13, 13));
        assert_eq!(Some(12), strategy.test_signed(13, 38));
        assert_eq!(Some(0), strategy.test_signed(13, -12));
        assert_eq!(Some(0), strategy.test_signed(13, -25));
    }
    #[test]
    fn bounds_strategy_unsigned_clip_works() {
        let strategy = BoundsStrategy::Clip;
        assert_eq!(Some(1), strategy.test_unsigned(13, 1));
        assert_eq!(Some(12), strategy.test_unsigned(13, 13));
        assert_eq!(Some(12), strategy.test_unsigned(13, 38));
    }
}
