use super::{FixedPoint, FixedPointContainer};

pub struct MultipliedFixedPoint<const P: isize, const R: isize, T>(T)
where
    T: FixedPointContainer;

impl<const P: isize, const R: isize, T> MultipliedFixedPoint<P, R, T>
where
    T: FixedPointContainer,
{
    pub fn truncate<const Q: isize>(&self) -> FixedPoint<Q, T> {
        FixedPoint::from_shifted(self.0, P + R)
    }
}

impl<const P: isize, const R: isize, T> core::ops::Mul<FixedPoint<R, T>> for FixedPoint<P, T>
where
    T: FixedPointContainer + core::ops::Mul<Output = T>,
{
    type Output = MultipliedFixedPoint<P, R, T>;

    fn mul(self, rhs: FixedPoint<R, T>) -> Self::Output {
        MultipliedFixedPoint(self.0 * rhs.0)
    }
}

pub struct DividedFixedPoint<const P: isize, const R: isize, T>(T)
where
    T: FixedPointContainer;

impl<const P: isize, const R: isize, T> DividedFixedPoint<P, R, T>
where
    T: FixedPointContainer,
{
    pub fn truncate<const Q: isize>(&self) -> FixedPoint<Q, T> {
        FixedPoint::from_shifted(self.0, P - R)
    }
}

impl<const P: isize, const R: isize, T> core::ops::Div<FixedPoint<R, T>> for FixedPoint<P, T>
where
    T: FixedPointContainer + core::ops::Div<Output = T>,
{
    type Output = DividedFixedPoint<P, R, T>;

    fn div(self, rhs: FixedPoint<R, T>) -> Self::Output {
        DividedFixedPoint(self.0 / rhs.0)
    }
}

impl<const P: isize, T> core::ops::Add<FixedPoint<P, T>> for FixedPoint<P, T>
where
    T: FixedPointContainer + core::ops::Add<T, Output = T>,
{
    type Output = FixedPoint<P, T>;

    fn add(self, rhs: FixedPoint<P, T>) -> Self::Output {
        FixedPoint::new(self.0 + rhs.0)
    }
}

impl<const P: isize, T> core::ops::Sub<FixedPoint<P, T>> for FixedPoint<P, T>
where
    T: FixedPointContainer + core::ops::Sub<T, Output = T>,
{
    type Output = FixedPoint<P, T>;

    fn sub(self, rhs: FixedPoint<P, T>) -> Self::Output {
        FixedPoint::new(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::FixedPoint;

    #[test]
    fn mul_works() {
        type Fx = FixedPoint<10, i32>;
        let a = Fx::try_from(0.5).unwrap();
        let b = Fx::try_from(-0.5).unwrap();
        let c: Fx = (a * b).truncate();
        assert_eq!(-0.25, c.to_f32_signed());
    }

    #[test]
    fn div_works() {
        type Fx = FixedPoint<10, i32>;
        let a = Fx::try_from(0.125).unwrap();
        let b = Fx::try_from(-0.5).unwrap();
        let c: Fx = (b / a).truncate();
        assert_eq!(-4.0, c.to_f32_signed());
    }

    #[test]
    fn add_works() {
        type Fx = FixedPoint<10, i32>;
        let a: Fx = 22.375.try_into().unwrap();
        let b: Fx = 22.125.try_into().unwrap();
        assert_eq!(44.5, (a + b).to_f32_signed());

        let a: Fx = 22.375.try_into().unwrap();
        let b: Fx = (-22.125).try_into().unwrap();
        assert_eq!(0.25, (a + b).to_f32_signed());
    }
}
