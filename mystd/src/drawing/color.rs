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
    pub(crate) from: HsvF,
    pub(crate) to: HsvF,
    pub(crate) t: f32,
    pub(crate) delta_t: f32,
    pub(crate) steps: usize,
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
pub(crate) mod tests_c {
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
