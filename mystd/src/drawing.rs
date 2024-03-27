use core::usize;

use crate::slice::slice2d::{traits::{MutSlice2dTrait, Slice2dTrait}, MutSlice2d};


#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

impl From<u32> for Rgba {
    fn from(value: u32) -> Rgba {
        let [r,g,b,a] = value.to_ne_bytes();
        Rgba { r, g, b, a }
    }
}

impl From<Bgra> for Rgba {
    fn from(value: Bgra) -> Rgba {
        Rgba { r: value.r, g: value.g, b: value.b, a: value.a }
    }
}

impl From<Rgba> for Bgra {
    fn from(value: Rgba) -> Bgra {
        Bgra { b: value.b, g: value.g, r: value.r, a: value.a }
    }
}

impl Into<u32> for Rgba {
    fn into(self) -> u32 {
        u32::from_ne_bytes([self.r, self.g, self.b, self.a])
    }
}

impl Rgba {
    pub const fn zero() -> Rgba {
        Rgba { r: 0, g: 0, b: 0, a: 0 }
    }

    pub const fn new_opaque(r: u8, g: u8, b: u8) -> Rgba {
        Rgba { r, g, b, a: 255 }
    }

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Rgba {
        Rgba { r, g, b, a }
    }

    pub fn new_f(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        if r > 1.0 || r.is_sign_negative() { panic!("Red value must be in [0.0..=1.0]"); }
        if g > 1.0 || g.is_sign_negative() { panic!("Green value must be in [0.0..=1.0]"); }
        if b > 1.0 || b.is_sign_negative() { panic!("Blue value must be in [0.0..=1.0]"); }
        if a > 1.0 || a.is_sign_negative() { panic!("Alpha value must be in [0.0..=1.0]"); }
        unsafe { Self::new_f_unchecked(b, g, r, a)}
    }

    pub fn new_f_saturating(mut r: f32, mut g: f32, mut b: f32, mut a: f32) -> Rgba {
        r = r.clamp(0.0, 1.0);
        g = g.clamp(0.0, 1.0);
        b = b.clamp(0.0, 1.0);
        a = a.clamp(0.0, 1.0);
        unsafe { Self::new_f_unchecked(r,g,b,a)}
    }

    pub unsafe fn new_f_unchecked(r: f32, g: f32, b: f32, a: f32) -> Rgba {
        Rgba { 
            r: (r * 255.0 + 0.5) as u8, 
            g: (g * 255.0 + 0.5) as u8,
            b: (b * 255.0 + 0.5) as u8,
            a: (a * 255.0 + 0.5) as u8 }
    }

    pub const fn from_u32(val: u32) -> Rgba {
        let [r,g,b,a] = val.to_ne_bytes();
        Rgba { r, g, b, a }
    }

    pub const fn from_u32be(val: u32) -> Rgba {
        let [r,g,b,a] = val.to_be_bytes();
        Rgba { r, g, b, a }
    }

    pub const fn from_u32le(val: u32) -> Rgba {
        let [r,g,b,a] = val.to_le_bytes();
        Rgba { r, g, b, a }
    }

    pub const fn into_u32(self) -> u32 {
        u32::from_ne_bytes([self.r, self.g, self.b, self.a])
    }

    pub const fn into_u32be(self) -> u32 {
        u32::from_be_bytes([self.r, self.g, self.b, self.a])
    }

    pub const fn into_u32le(self) -> u32 {
        u32::from_le_bytes([self.r, self.g, self.b, self.a])
    }

    pub const fn into_bgra(self) -> Bgra {
        Bgra { b: self.r, g: self.g, r: self.b, a: self.a }
    }

    pub fn ref_array_to_u32<const N: usize>(array: &[Self; N]) -> &[u32; N] {
        unsafe { &*array.as_ptr().cast() }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bgra {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8
}

impl Bgra {
    pub const fn zero() -> Bgra {
        Bgra { b: 0, g: 0, r: 0, a: 0 }
    }

    pub const fn new_opaque(b: u8, g: u8, r: u8) -> Bgra {
        Bgra { r, g, b, a: 255 }
    }

    pub const fn new(b: u8, g: u8, r: u8, a: u8) -> Bgra {
        Bgra { r, g, b, a }
    }

    pub fn new_f(b: f32, g: f32, r: f32, a: f32) -> Bgra {
        if b > 1.0 || b.is_sign_negative() { panic!("Blue value must be in [0.0..=1.0]"); }
        if g > 1.0 || g.is_sign_negative() { panic!("Green value must be in [0.0..=1.0]"); }
        if r > 1.0 || r.is_sign_negative() { panic!("Red value must be in [0.0..=1.0]"); }
        if a > 1.0 || a.is_sign_negative() { panic!("Alpha value must be in [0.0..=1.0]"); }
        unsafe { Self::new_f_unchecked(b, g, r, a)}
    }

    pub fn new_f_saturating(mut b: f32, mut g: f32, mut r: f32, mut a: f32) -> Bgra {
        b = b.clamp(0.0, 1.0);
        g = g.clamp(0.0, 1.0);
        r = r.clamp(0.0, 1.0);
        a = a.clamp(0.0, 1.0);
        unsafe { Self::new_f_unchecked(b, g, r, a)}
    }

    pub unsafe fn new_f_unchecked(b: f32, g: f32, r: f32, a: f32) -> Bgra {
        Bgra { 
            b: (b * 255.0 + 0.5) as u8,
            g: (g * 255.0 + 0.5) as u8,
            r: (r * 255.0 + 0.5) as u8, 
            a: (a * 255.0 + 0.5) as u8 }
    }

    pub const fn from_u32(val: u32) -> Bgra {
        let [b,g,r,a] = val.to_ne_bytes();
        Bgra { b, g, r, a }
    }

    pub const fn from_u32be(val: u32) -> Bgra {
        let [b,g,r,a] = val.to_be_bytes();
        Bgra { b, g, r, a }
    }

    pub const fn from_u32le(val: u32) -> Bgra {
        let [b,g,r,a] = val.to_le_bytes();
        Bgra { b, g, r, a }
    }

    pub const fn into_u32(self) -> u32 {
        u32::from_ne_bytes([self.b, self.g, self.r, self.a])
    }

    pub const fn into_u32be(self) -> u32 {
        u32::from_be_bytes([self.b, self.g, self.r, self.a])
    }

    pub const fn into_u32le(self) -> u32 {
        u32::from_le_bytes([self.b, self.g, self.r, self.a])
    }

    pub const fn into_rgba(self) -> Rgba {
        Rgba { r: self.r, g: self.g, b: self.b, a: self.a }
    }

    pub fn ref_array_to_u32<const N: usize>(array: &[Self; N]) -> &[u32; N] {
        unsafe { &*array.as_ptr().cast() }
    }
}

impl From<u32> for Bgra {
    fn from(value: u32) -> Bgra {
        let [b,g,r,a] = value.to_ne_bytes();
        Bgra { b, g, r, a }
    }
}

impl Into<u32> for Bgra {
    fn into(self) -> u32 {
        u32::from_ne_bytes([self.b, self.g, self.r, self.a])
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct RgbF {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct HsvF {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl From<RgbF> for Bgra {
    fn from(value: RgbF) -> Bgra {
        Bgra::new_f_saturating(value.b, value.g, value.r, 1.0)
    }
}

impl From<RgbF> for Rgba {
    fn from(value: RgbF) -> Rgba {
        Rgba::new_f_saturating(value.r, value.g, value.b, 1.0)
    }
}

impl From<Rgba> for RgbF {
    fn from(value: Rgba) -> RgbF {
        RgbF { 
            r: value.r as f32 / 255.0, 
            g: value.g as f32 / 255.0, 
            b: value.b as f32 / 255.0, 
        }
    }
}

impl From<Bgra> for RgbF {
    fn from(value: Bgra) -> RgbF {
        RgbF { 
            r: value.r as f32 / 255.0, 
            g: value.g as f32 / 255.0, 
            b: value.b as f32 / 255.0, 
        }
    }
}


impl From<HsvF> for RgbF {
    fn from(mut value: HsvF) -> Self {
        value = value.clamped_s_v();
        let s = value.s;
        let v = value.v;
        if s == 0.0 {
            // gray
            return RgbF { r: v, g: v, b: v };
        }
        value = value.normalized_tint();
        let h = value.h * 6.0;
        let h_i = h as i32;
        let f = h - h_i as f32;

        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));
        match h_i {
            0 | 6 => RgbF { r: v, g: t, b: p },
            1 => RgbF { r: q, g: v, b: p },
            2 => RgbF { r: p, g: v, b: t },
            3 => RgbF { r: p, g: q, b: v },
            4 => RgbF { r: t, g: p, b: v },
            5 => RgbF { r: v, g: p, b: q },
            _ => panic!()
        }
    }
}

impl From<RgbF> for HsvF {
    fn from(value: RgbF) -> Self {
        let r = value.r.clamp(0.0, 1.0);
        let g = value.g.clamp(0.0, 1.0);
        let b = value.b.clamp(0.0, 1.0);
        let max = r.max(g.max(b));
        let min = r.min(g.min(b));
        if max == 0.0 {
            // black
            HsvF { h: 0.0, s: 0.0, v: 0.0 }
        } else {
            let v = max;
            if max == min {
                // gray
                HsvF { h: 0.0, s: 0.0, v } 
            } else {
                let s = (max - min) / max;
                let factor = 1.0 / (max - min);
                let h = 
                    if max == r {
                        (g - b) * factor 
                    } else if max == g {
                        2.0 + (b - r) * factor
                    } else {
                        // max == b
                        4.0 + (r - g) * factor
                    };
                HsvF { h: h / 6.0, s, v}
            }
        }
    }
}

impl RgbF {
    pub const BLACK: RgbF = RgbF { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE: RgbF = RgbF { r: 1.0, g: 1.0, b: 1.0 };
    pub const GRAY: RgbF = RgbF { r: 0.5, g: 0.5, b: 0.5 };

}

impl HsvF {
    pub const BLACK: HsvF = HsvF { h: 0.0, s: 0.0, v: 0.0 };
    pub const WHITE: HsvF = HsvF { h: 0.0, s: 0.0, v: 1.0 };
    pub const GRAY: HsvF = HsvF { h: 0.0, s: 0.0, v: 0.5 };
    pub const RED: HsvF = HsvF { h: 0.0, s: 1.0, v: 1.0 };
    pub const ORANGE: HsvF = HsvF { h: 0.083333333333333, s: 1.0, v: 1.0 };
    pub const YELLOW: HsvF = HsvF { h: 0.166666666666667, s: 1.0, v: 1.0 };
    pub const GREEN: HsvF = HsvF { h: 0.25, s: 1.0, v: 1.0 };
    pub const BRIGHT_GREEN: HsvF = HsvF { h: 0.33333333333333, s: 1.0, v: 1.0 };
    pub const SPRING_GREEN: HsvF = HsvF { h: 0.416666666666667, s: 1.0, v: 1.0 };
    pub const CYAN: HsvF = HsvF { h: 0.5, s: 1.0, v: 1.0 };
    pub const GREENISH_BLUE: HsvF = HsvF { h: 0.583333333333333, s: 1.0, v: 1.0 };
    pub const BLUE: HsvF = HsvF { h: 0.666666666666667, s: 1.0, v: 1.0 };
    pub const VIOLET: HsvF = HsvF { h: 0.75, s: 1.0, v: 1.0 };
    pub const MAGENTA: HsvF = HsvF { h: 0.833333333333333, s: 1.0, v: 1.0 };
    pub const BLUEISH_RED: HsvF = HsvF { h: 0.916666666666667, s: 1.0, v: 1.0 };
    pub const RED_2: HsvF = HsvF { h: 1.0, s: 1.0, v: 1.0 };
    pub const ORANGE_2: HsvF = HsvF { h: 1.083333333333333, s: 1.0, v: 1.0 };
    pub const YELLOW_2: HsvF = HsvF { h: 1.166666666666667, s: 1.0, v: 1.0 };
    pub const GREEN_2: HsvF = HsvF { h: 1.25, s: 1.0, v: 1.0 };
    pub const BRIGHT_GREEN_2: HsvF = HsvF { h: 1.33333333333333, s: 1.0, v: 1.0 };
    pub const SPRING_GREEN_2: HsvF = HsvF { h: 1.416666666666667, s: 1.0, v: 1.0 };
    pub const CYAN_2: HsvF = HsvF { h: 1.5, s: 1.0, v: 1.0 };
    pub const GREENISH_BLUE_2: HsvF = HsvF { h: 1.583333333333333, s: 1.0, v: 1.0 };
    pub const BLUE_2: HsvF = HsvF { h: 1.666666666666667, s: 1.0, v: 1.0 };
    pub const VIOLET_2: HsvF = HsvF { h: 1.75, s: 1.0, v: 1.0 };
    pub const MAGENTA_2: HsvF = HsvF { h: 1.833333333333333, s: 1.0, v: 1.0 };
    pub const BLUEISH_RED_2: HsvF = HsvF { h: 1.916666666666667, s: 1.0, v: 1.0 };

    pub fn lerp(self, to: HsvF, steps: usize) -> HsvFLerp {
        assert!(steps > 2, "Can't interpolate with less than 3 steps, since first and last are the start and endpoint");
        HsvFLerp { from: self, to, t: 0.0, delta_t: 1.0 / (steps - 1) as f32, steps: steps }
    }

    pub fn clamped_s_v(self) -> Self {
        Self { s: self.s.clamp(0.0, 1.0), v: self.v.clamp(0.0, 1.0), ..self }
    }

    pub fn normalized_tint(self) -> Self {
        let mut h = self.h - (self.h as i32 as f32);
        if h.is_sign_negative() {
            h += 1.0;
        }
        Self { h, ..self }
    }

    // moves h by the given amount
    pub fn tint_added(self, value: f32) -> Self {
        Self { h: self.h + value, ..self }
    }

    // darkens for values < 1.0
    pub fn dimmed_by(self, value: f32) -> Self {
        Self { v: (self.v * value).clamp(0.0, 1.0), ..self }
    }

    // desaturates for values < 1.0
    pub fn lifted_by(self, value: f32) -> Self {
        Self { s: (self.s * value).clamp(0.0, 1.0), ..self }
    }
}

pub struct HsvFLerp {
    from: HsvF,
    to: HsvF,
    t: f32,
    delta_t: f32,
    steps: usize,
}

impl Iterator for HsvFLerp {
    type Item = HsvF;

    fn next(&mut self) -> Option<Self::Item> {
        if self.steps == 0 {
            None
        } else if self.steps == 1 {
            self.steps -= 1;
            Some(self.to)
        } else {
            let t_rem = 1.0 - self.t;
            let h = self.from.h * t_rem + self.to.h * self.t;
            let s = self.from.s * t_rem + self.to.s * self.t;
            let v = self.from.v * t_rem + self.to.v * self.t;
            self.steps -= 1;
            self.t += self.delta_t;
            Some(HsvF { h, s, v })
        }
    }
}



#[cfg(test)]
mod tests_c {
    use super::*;

    #[test]
    fn hsv_color_lerp_works() {
        let mut iter = HsvF::BLACK.lerp(HsvF::WHITE, 3);
        assert_eq!(HsvF::BLACK, iter.next().expect("should iter"));
        assert_eq!(HsvF::GRAY, iter.next().expect("should iter"));
        assert_eq!(HsvF::WHITE, iter.next().expect("should iter"));
        assert_eq!(None, iter.next());

        let mut iter = HsvF::BLACK.lerp(HsvF::WHITE, 5);
        assert_eq!(HsvF::BLACK, iter.next().expect("should iter"));
        assert_eq!(HsvF{ v: 0.25, ..Default::default() }, iter.next().expect("should iter"));
        assert_eq!(HsvF{ v: 0.50, ..Default::default() }, iter.next().expect("should iter"));
        assert_eq!(HsvF{ v: 0.75, ..Default::default() }, iter.next().expect("should iter"));
        assert_eq!(HsvF::WHITE, iter.next().expect("should iter"));
        assert_eq!(None, iter.next());
    }

    #[test]
    fn hsv_to_rgb_works() {
        assert_eq!(RgbF::BLACK, HsvF::BLACK.into());
        assert_eq!(RgbF::WHITE, HsvF::WHITE.into());
        assert_eq!(RgbF::GRAY, HsvF::GRAY.into());   
    }

    #[test]
    fn rgbf_to_rgba_works() {
        assert_eq!(Rgba::new_opaque(255, 255, 255), RgbF::WHITE.into());
        assert_eq!(Rgba::new_opaque(128, 128, 128), RgbF::GRAY.into());
        assert_eq!(Rgba::new_opaque(0, 0, 0), RgbF::BLACK.into());
    }
}



pub struct PixelCanvas<'a, T> {
    data: &'a mut MutSlice2d<'a, T>,
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

    fn check_bounds(&self, x: usize, y: usize) -> Result<(), CanvasAccessError> {
        if x >= self.data.width() {
            Err(CanvasAccessError::OverflowX)
        } else if y >= self.data.height() {
            Err(CanvasAccessError::OverflowY)
        } else {
            Ok(())
        }
    }

    fn lin(&self, x: usize, y: usize) -> usize {
        x + y * self.data.pitch()
    }

    fn bounded_coords(&self, (x, y): (isize, isize), horizontal_bounds: BoundsStrategy, vertical_bounds: BoundsStrategy) -> Option<(usize, usize)> {
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
        let mut start_line: usize = match range.start_bound() {
            core::ops::Bound::Included(y) => *y,
            core::ops::Bound::Excluded(y) => y + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let mut end_line: usize = match range.end_bound() {
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
        const MASK: [u8; 16] = [
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
        const MASK: [u8; 16] = [
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
        const MASK: [u8; 16] = [
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
mod tests {
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