
/// calculates the greatest common divisor of two numbers
pub fn gcd<T>(mut a: T, mut b: T) -> T
where
    T: core::ops::Rem<T, Output = T> + PartialEq + Copy,
{
    let zero = a % a;
    while b != zero {
        let remainder = a % b;
        a = core::mem::replace(&mut b, remainder);
    }
    a
}

#[derive(Copy, Clone)]
pub struct Fract<T> { numerator: T, denominator: T }

impl<T> core::ops::Mul<T> for Fract<T> where T: core::ops::Mul<Output = T> {
    type Output = Fract<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Fract::new(self.numerator * rhs, self.denominator)
    }
}

impl<T> core::ops::Div<T> for Fract<T> where T: core::ops::Mul<Output = T> {
    type Output = Fract<T>;

    fn div(self, rhs: T) -> Self::Output {
        Fract::new(self.numerator, self.denominator * rhs)
    }
}

impl<T> core::ops::Mul for Fract<T> where T: core::ops::Mul {
    type Output = Fract<<T as core::ops::Mul>::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        Fract::new(self.numerator * rhs.numerator, self.denominator * rhs.denominator)
    }
}

impl<T> core::ops::Div for Fract<T> where T: core::ops::Mul {
    type Output = Fract<<T as core::ops::Mul>::Output>;

    fn div(self, rhs: Self) -> Self::Output {
        Fract::new(self.numerator * rhs.denominator, self.denominator * rhs.numerator)
    }
}

impl<T> Fract<T> {
    pub fn new(numerator: T, denominator: T) -> Self {
        Self { numerator, denominator }
    }

    pub fn reciprocal(self) -> Self
    where
        T: Copy,
    {
        Self{numerator: self.denominator, denominator: self.numerator}
    }

    pub fn reciprocate(&mut self) {
        core::mem::swap(&mut self.numerator, &mut self.denominator);
    }
}

impl<T> Fract<T>
where
    T: core::ops::Rem<T, Output = T> + PartialEq + Copy + core::ops::Div<Output = T>,
{
    pub fn reduced(self) -> Self {
        let d = gcd(self.numerator, self.denominator);
        self.reduce_by(d)
    }

    pub fn reduce_by(self, value: T) -> Self {
        Self::new(self.numerator / value, self.denominator / value)
    }
}

impl<T, U, R> Fract<T>
where
    T: core::ops::Mul<Output = U>,
    U: core::ops::Div<T, Output = R>
{
    pub fn mul(self, rhs: T) -> R {
        (rhs * self.numerator) / self.denominator
    }

    pub fn dividing(self, rhs: T) -> R {
        (rhs * self.denominator) / self.numerator
    }

}

impl<T> core::fmt::Display for Fract<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.numerator, self.denominator)
    }
}

