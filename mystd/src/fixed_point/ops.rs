
pub struct Multiply<T, const P: i32, const Q: i32>(pub T, pub T);
pub struct Divide<T, const P: i32, const Q: i32>(pub T, pub T);

// pub struct DividedFixedPoint<const P: isize, const R: isize, T>(T, T)
// where
//     T: FixedPointContainer + core::ops::Div<Output = T>;

// impl<const P: isize, const R: isize, T> DividedFixedPoint<P, R, T>
// where
//     T: FixedPointContainer + core::ops::Div<Output = T>,
// {
//     pub fn truncate<const Q: isize>(&self) -> FixedPoint<Q, T> {
//         FixedPoint::from_shifted(self.0 / self.1, P - R)
//     }

//     pub fn rounded_mid<const Q: isize>(&self) -> FixedPoint<Q, T> {
//         self.rounded_right(Q >> 1)
//     }

//     pub fn rounded_right<const Q: isize>(&self, precision: isize) -> FixedPoint<Q, T> {
//         FixedPoint::from_shifted(self.0 / self.1.signed_shift(R, precision), P - precision)
//     }
// }

// impl<const P: isize, const R: isize, T> core::ops::Div<FixedPoint<R, T>> for FixedPoint<P, T>
// where
//     T: FixedPointContainer + core::ops::Div<Output = T>,
// {
//     type Output = DividedFixedPoint<P, R, T>;

//     fn div(self, rhs: FixedPoint<R, T>) -> Self::Output {
//         DividedFixedPoint(self.0, rhs.0)
//     }
// }

// impl<const P: isize, T> core::ops::Div<T> for FixedPoint<P, T>
// where
//     T: FixedPointContainer + core::ops::Div<Output = T>,
// {
//     type Output = Self;

//     fn div(self, rhs: T) -> Self::Output {
//         FixedPoint::new(self.0 / rhs)
//     }
// }

// impl<const P: isize, T> core::ops::Add<FixedPoint<P, T>> for FixedPoint<P, T>
// where
//     T: FixedPointContainer + core::ops::Add<T, Output = T>,
// {
//     type Output = FixedPoint<P, T>;

//     fn add(self, rhs: FixedPoint<P, T>) -> Self::Output {
//         FixedPoint::new(self.0 + rhs.0)
//     }
// }

// impl<const P: isize, T> core::ops::Sub<FixedPoint<P, T>> for FixedPoint<P, T>
// where
//     T: FixedPointContainer + core::ops::Sub<T, Output = T>,
// {
//     type Output = FixedPoint<P, T>;

//     fn sub(self, rhs: FixedPoint<P, T>) -> Self::Output {
//         FixedPoint::new(self.0 - rhs.0)
//     }
// }

// impl<const P: isize, T> core::ops::Neg for FixedPoint<P, T>
// where
//     T: FixedPointContainer + core::ops::Neg<Output = T>,
// {
//     type Output = FixedPoint<P, T>;

//     fn neg(self) -> Self::Output {
//         FixedPoint::new(-self.0)
//     }
// }

#[cfg(test)]
mod tests {
    use crate::fixed_point::FxS32;

    type Fx = FxS32<16>;
    type Fx2 = FxS32<24>;

    #[test]
    fn mul_works() {
        let a = Fx::try_from(0.5).unwrap();
        let b = Fx::try_from(-0.5).unwrap();
        let c: Fx = (a * b).resolve();
        assert_eq!(-0.25, c.into());
    }

    #[test]
    fn div_works() {
        let a = Fx::try_from(0.125).unwrap();
        let b = Fx::try_from(-0.5).unwrap();
        let c: Fx = (b / a).resolve();
        assert_eq!(-4.0, c.into());
        let c: Fx = (a / b).resolve();
        assert_eq!(0.0, c.into());
    }

    #[test]
    fn add_assign_works() {
        let mut a: Fx = 22.375.try_into().unwrap();
        let b: Fx2 = 22.125.try_into().unwrap();
        a += b;
        assert_eq!(44.5, a.into());

        let mut a: Fx2 = 22.375.try_into().unwrap();
        let b: Fx = (-22.125).try_into().unwrap();
        a += b;
        assert_eq!(0.25, a.into());
    }

    #[test]
    fn sub_assign_works() {
        let mut a: Fx = 22.375.try_into().unwrap();
        let b: Fx2 = 22.125.try_into().unwrap();
        a -= b;
        assert_eq!(0.25, a.into());

        let mut a: Fx2 = 22.375.try_into().unwrap();
        let b: Fx = (-22.125).try_into().unwrap();
        a -= b;
        assert_eq!(44.5, a.into());
    }


    // #[test]
    // fn add_works() {
    //     let a: Fx = 22.375.try_into().unwrap();
    //     let b: Fx = 22.125.try_into().unwrap();
    //     assert_eq!(44.5, (a + b).to_f32_signed());

    //     let a: Fx = 22.375.try_into().unwrap();
    //     let b: Fx = (-22.125).try_into().unwrap();
    //     assert_eq!(0.25, (a + b).to_f32_signed());
    // }

    // #[test]
    // fn sub_works() {
    //     let a: Fx = 22.375.try_into().unwrap();
    //     let b: Fx = 22.125.try_into().unwrap();
    //     assert_eq!(0.25, (a - b).to_f32_signed());

    //     let a: Fx = 22.375.try_into().unwrap();
    //     let b: Fx = (-22.125).try_into().unwrap();
    //     assert_eq!(44.5, (a - b).to_f32_signed());
    // }

    // #[test]
    // fn neg_works() {
    //     let a: Fx = 22.375.try_into().unwrap();
    //     assert_eq!(-22.375, (-a).to_f32_signed());
    // }
}
