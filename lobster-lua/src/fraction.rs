use std::ops::Add;

use serde::Serialize;

#[derive(PartialEq, Eq, Clone, Copy, Serialize)]
pub struct Fraction {
    numerator: i64,
    denominator: i64,
}

impl std::fmt::Debug for Fraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl std::fmt::Display for Fraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl Fraction {
    pub const fn new_unreduced(numerator: i64, denominator: i64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn new(numerator: i64, denominator: i64) -> Self {
        let mut this = Self {
            numerator,
            denominator,
        };
        this.reduce();
        this
    }

    fn reduce(&mut self) {
        assert_ne!(self.denominator, 0);

        let mut a = self.numerator;
        let mut b = self.denominator;

        while b != 0 {
            let c = b;
            b = a % b;
            a = c
        }

        assert_eq!(self.denominator % a, 0);
        assert_eq!(self.numerator % a, 0);
        self.denominator /= a;
        self.numerator /= a;
    }
}

impl std::ops::Add for Fraction {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Fraction::new(
            self.numerator * rhs.denominator + self.denominator * rhs.numerator,
            self.denominator * rhs.denominator,
        )
    }
}

impl std::ops::Neg for Fraction {
    type Output = Self;

    // WTF mut => self is copied and just the local self is mut then or something
    fn neg(mut self) -> Self::Output {
        self.numerator *= -1;
        self
    }
}

impl std::ops::Sub for Fraction {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + -rhs
    }
}

impl std::ops::Mul for Fraction {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.numerator * rhs.numerator, self.denominator * rhs.denominator)
    }
}

impl std::ops::Div for Fraction {
    type Output = Self;
    
    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.numerator * rhs.denominator, self.denominator * rhs.numerator)
    }
}

#[cfg(test)]
mod tests {
    use super::Fraction;

    #[test]
    fn reduce_test() {
        fn check_reduce(a: i64, b: i64, a2: i64, b2: i64) {
            let f = Fraction::new(a, b);
            assert_eq!(f.numerator, a2);
            assert_eq!(f.denominator, b2);
        }

        check_reduce(0, 3, 0, 1);

        check_reduce(2, 3, 2, 3);
        check_reduce(20, 30, 2, 3);

        check_reduce(-2, -3, 2, 3);
        check_reduce(-2, 3, -2, 3);
        check_reduce(2, -3, -2, 3);
        check_reduce(-20, -30, 2, 3);
    }

    #[test]
    #[should_panic]
    fn reduce_panic_test() {
        Fraction::new(3, 0);
    }

    #[test]
    fn add_test() {
        let a = Fraction::new(73, 111678);
        let b = Fraction::new(1, 1);

        let res = a + b;
        let expected_num = 111678 + 73;
        let expected_denom = 111678;

        assert_eq!(res.numerator, expected_num);
        assert_eq!(res.denominator, expected_denom);
    }

    #[test]
    fn add_test_1_6_1_3() {
        let a = Fraction::new(1, 6);
        let b = Fraction::new(1, 3);

        let res = a + b;

        assert_eq!(res.numerator, 1);
        assert_eq!(res.denominator, 2);
    }

    #[test]
    fn mul_test() {
        let a = Fraction::new(1,2);
        let b = Fraction::new(2,3);

        let res = a * b;

        assert_eq!(res.numerator, 1);
        assert_eq!(res.denominator, 3)
    }
}
