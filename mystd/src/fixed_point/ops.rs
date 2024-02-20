use super::{FixedPoint, FixedPointContainer};

pub struct MultipliedFixedPoint<const P: isize, const R: isize, T: FixedPointContainer<T>> (T);

impl<const P: isize, const R: isize, T: FixedPointContainer<T>> MultipliedFixedPoint<P,R,T> {
    pub fn truncate<const Q: isize>(&self) -> FixedPoint<Q, T> {
        FixedPoint::from_shifted(self.0, P + R)
    }
}

impl<const P: isize, const R: isize, T: FixedPointContainer<T>> core::ops::Mul<FixedPoint<R, T>> for FixedPoint<P, T> {
    type Output = MultipliedFixedPoint<P, R, T>;

    fn mul(self, rhs: FixedPoint<R, T>) -> Self::Output {
        MultipliedFixedPoint(self.0 * rhs.0)
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
}