//! A fast and efficient integer implementation designed to work
//! under low-resource environments. This originally belongs to the Hxro Network Dexterity protocol
//! but adopted here to improve performance and reduce resource consumption

// standard library imports
use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use borsh::{BorshDeserialize, BorshSerialize};

// other
use {
    bytemuck::{Pod, Zeroable},
    serde::{self, Deserialize, Serialize},
};

use crate::error::MathError;
use crate::fractional::{Fractional, ZERO_FRAC};

pub const F32_POW_LOOKUP: [f32; 16] = [
    1.0,
    10.0,
    100.0,
    1_000.0,
    10_000.0,
    100_000.0,
    1_000_000.0,
    10_000_000.0,
    100_000_000.0,
    1_000_000_000.0,
    10_000_000_000.0,
    100_000_000_000.0,
    1_000_000_000_000.0,
    10_000_000_000_000.0,
    100_000_000_000_000.0,
    1_000_000_000_000_000.0,
];

pub const F64_POW_LOOKUP: [f64; 16] = [
    1.0,
    10.0,
    100.0,
    1_000.0,
    10_000.0,
    100_000.0,
    1_000_000.0,
    10_000_000.0,
    100_000_000.0,
    1_000_000_000.0,
    10_000_000_000.0,
    100_000_000_000.0,
    1_000_000_000_000.0,
    10_000_000_000_000.0,
    100_000_000_000_000.0,
    1_000_000_000_000_000.0,
];

pub const I128_POW_LOOKUP: [i128; 16] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
];

pub const FAST_INT_DIGITS: i64 = 6;
pub const FAST_INT_CONVERSION: i128 = 1_000_000;
pub const FAST_INT_CONVERSION_F32: f32 = 1_000_000_f32;
pub const FAST_INT_CONVERSION_F64: f64 = 1_000_000_f64;
pub const ZERO_FAST_INT: FastInt = FastInt { value: 0_i128 };
pub const TWO_FAST_INT: FastInt = FastInt {
    value: 2_000_000_i128,
};
pub const NEGATIVE_ERROR_ROUND_TO_ZERO_FAST_INT: FastInt = FastInt { value: -1000_i128 };
pub const MAXIMUM_NEGATIVE_FAST_INT: FastInt = FastInt { value: -1_i128 };
pub const MINIMUM_POSITIVE_FAST_INT: FastInt = FastInt { value: 1_i128 };

#[repr(C)]
#[derive(
    Eq,
    Clone,
    Copy,
    Zeroable,
    Pod,
    Debug,
    Serialize,
    Deserialize,
    BorshSerialize,
    BorshDeserialize,
    schemars::JsonSchema,
    sov_modules_api::macros::UniversalWallet,
)] // added a PartialEq here considering the OG codebase does not implement it and Eq likes to be with PartialEq :P
pub struct FastInt {
    pub value: i128,
}

impl Neg for FastInt {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { value: -self.value }
    }
}

impl Add for FastInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

impl AddAssign for FastInt {
    fn add_assign(&mut self, other: Self) {
        *self = self.add(other);
    }
}

impl Sub for FastInt {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            value: self.value - other.value,
        }
    }
}

impl SubAssign for FastInt {
    fn sub_assign(&mut self, other: Self) {
        *self = self.sub(other);
    }
}

impl Mul for FastInt {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        if other == ZERO_FAST_INT || self == ZERO_FAST_INT {
            return ZERO_FAST_INT;
        };
        let result = Self {
            value: self.value * other.value / FAST_INT_CONVERSION,
        };
        assert!(result != ZERO_FAST_INT);
        result
    }
}

impl MulAssign for FastInt {
    fn mul_assign(&mut self, other: Self) {
        *self = self.mul(other);
    }
}

impl Div for FastInt {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        (self.to_f64() / other.to_f64()).into()
    }
}

impl DivAssign for FastInt {
    fn div_assign(&mut self, other: Self) {
        *self = self.div(other);
    }
}

impl PartialEq for FastInt {
    fn eq(&self, other: &Self) -> bool {
        match self.partial_cmp(other) {
            Some(Ordering::Equal) => true,
            _ => false,
        }
    }
}

impl PartialOrd for FastInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl From<i64> for FastInt {
    fn from(x: i64) -> Self {
        let result = Self {
            value: (x as i128) * FAST_INT_CONVERSION,
        };
        // assert!(!(x != 0_i64 && result == ZERO_FAST_INT));
        result
    }
}

impl From<f64> for FastInt {
    fn from(x: f64) -> Self {
        let result = Self {
            value: (x * FAST_INT_CONVERSION_F64).round() as i128,
        };
        // assert!(!(x != 0_f64 && result == ZERO_FAST_INT));
        result
    }
}

impl From<f32> for FastInt {
    fn from(x: f32) -> Self {
        let result = Self {
            value: (x * FAST_INT_CONVERSION_F32).round() as i128,
        };
        // assert!(!(x != 0_f32 && result == ZERO_FAST_INT));
        result
    }
}

impl From<Fractional> for FastInt {
    fn from(frac: Fractional) -> Self {
        let exp_diff = FAST_INT_DIGITS - (frac.exp as i64);
        let result = if exp_diff < 0 {
            FastInt {
                value: (frac.m as i128) / I128_POW_LOOKUP[exp_diff.abs() as usize],
            }
        } else if exp_diff > 0 {
            FastInt {
                value: (frac.m as i128) * I128_POW_LOOKUP[exp_diff as usize],
            }
        } else {
            FastInt {
                value: frac.m as i128,
            }
        };
        // assert!(!(frac != ZERO_FRAC && result == ZERO_FAST_INT));
        result
    }
}

impl Mul<i64> for FastInt {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            value: self.value * (rhs as i128),
        }
    }
}

impl Add<i64> for FastInt {
    type Output = Self;
    fn add(self, rhs: i64) -> Self::Output {
        Self {
            value: self.value + (rhs as i128),
        }
    }
}

impl Add<FastInt> for i64 {
    type Output = FastInt;
    fn add(self, rhs: FastInt) -> Self::Output {
        rhs + self
    }
}

impl Mul<FastInt> for i64 {
    type Output = FastInt;
    fn mul(self, rhs: FastInt) -> Self::Output {
        rhs * self
    }
}

impl Display for FastInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}

impl FastInt {
    pub fn to_f32(&self) -> f32 {
        (self.value as f32) / FAST_INT_CONVERSION_F32
    }

    pub fn to_f64(&self) -> f64 {
        (self.value as f64) / FAST_INT_CONVERSION_F64
    }

    pub fn abs(&self) -> Self {
        Self {
            value: self.value.abs(),
        }
    }

    pub fn max(&self, other: FastInt) -> Self {
        match *self > other {
            true => *self,
            false => other,
        }
    }

    pub fn min(&self, other: FastInt) -> Self {
        match *self < other {
            true => *self,
            false => other,
        }
    }

    pub fn to_frac(&self) -> std::result::Result<Fractional, MathError> {
        let mut m = self.value;
        let mut exp = FAST_INT_DIGITS as u64;

        let _ = Fractional::reduce_from_i128(&mut m, &mut exp)
            .map_err(|_| MathError::NumericalOverflow);
        Ok(Fractional::new(m as i64, exp))
    }

    pub fn mul_clamp_to_tick(self, other: Self) -> Self {
        match self.checked_mul(other) {
            Ok(x) => x,
            Err(_) => match (self.value > 0, other.value > 0) {
                (true, true) => MINIMUM_POSITIVE_FAST_INT,
                (false, false) => MINIMUM_POSITIVE_FAST_INT,
                _ => MAXIMUM_NEGATIVE_FAST_INT,
            },
        }
    }

    pub fn checked_mul(self, other: Self) -> std::result::Result<Self, MathError> {
        let result = self.mul_zero_okay(other);
        if result != ZERO_FAST_INT {
            Ok(result)
        } else {
            if self == ZERO_FAST_INT || other == ZERO_FAST_INT {
                Ok(ZERO_FAST_INT)
            } else {
                Err(MathError::FastIntCoercionToZero.into())
            }
        }
    }

    pub fn mul_zero_okay(self, other: Self) -> Self {
        if other == ZERO_FAST_INT || self == ZERO_FAST_INT {
            return ZERO_FAST_INT;
        };
        Self {
            value: self.value * other.value / FAST_INT_CONVERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_int_addition() {
        let a = FastInt::from(1_i64);
        let b = FastInt::from(2_i64);
        let result = a + b;
        assert_eq!(result, FastInt::from(3_i64));
    }

    #[test]
    fn test_fast_int_subtraction() {
        let a = FastInt::from(3_i64);
        let b = FastInt::from(1_i64);
        let result = a - b;
        assert_eq!(result, FastInt::from(2_i64));
    }

    #[test]
    fn test_fast_int_multiplication() {
        let a = FastInt::from(2_i64);
        let b = FastInt::from(3_i64);
        let result = a * b;
        assert_eq!(result, FastInt::from(6_i64));
    }

    #[test]
    fn test_fast_int_division() {
        let a = FastInt::from(6_i64);
        let b = FastInt::from(3_i64);
        let result = a / b;
        assert_eq!(result, FastInt::from(2_i64));
    }

    #[test]
    fn test_fast_int_add_assign() {
        let mut a = FastInt::from(1_i64);
        let b = FastInt::from(2_i64);
        a += b;
        assert_eq!(a, FastInt::from(3_i64));
    }

    #[test]
    fn test_fast_int_sub_assign() {
        let mut a = FastInt::from(3_i64);
        let b = FastInt::from(1_i64);
        a -= b;
        assert_eq!(a, FastInt::from(2_i64));
    }

    #[test]
    fn test_fast_int_mul_assign() {
        let mut a = FastInt::from(2_i64);
        let b = FastInt::from(3_i64);
        a *= b;
        assert_eq!(a, FastInt::from(6_i64));
    }

    #[test]
    fn test_fast_int_div_assign() {
        let mut a = FastInt::from(6_i64);
        let b = FastInt::from(3_i64);
        a /= b;
        assert_eq!(a, FastInt::from(2_i64));
    }

    #[test]
    fn test_fast_int_from_i64() {
        let a = FastInt::from(123_i64);
        assert_eq!(a.value, 123 * FAST_INT_CONVERSION);
    }

    #[test]
    fn test_fast_int_from_f32() {
        let a = FastInt::from(123.456_f32);
        assert_eq!(a.value, (123.456 * FAST_INT_CONVERSION_F32).round() as i128);
    }

    #[test]
    fn test_fast_int_from_f64() {
        let a = FastInt::from(123.456_f64);
        assert_eq!(a.value, (123.456 * FAST_INT_CONVERSION_F64).round() as i128);
    }

    #[test]
    fn test_fast_int_negation() {
        let a = FastInt::from(123_i64);
        let result = -a;
        assert_eq!(result, FastInt::from(-123_i64));
    }

    #[test]
    fn test_fast_int_equality() {
        let a = FastInt::from(123_i64);
        let b = FastInt::from(123_i64);
        assert_eq!(a, b);
    }

    #[test]
    fn test_fast_int_inequality() {
        let a = FastInt::from(123_i64);
        let b = FastInt::from(456_i64);
        assert_ne!(a, b);
    }

    #[test]
    fn test_fast_int_max() {
        let a = FastInt::from(123_i64);
        let b = FastInt::from(456_i64);
        assert_eq!(a.max(b), b);
    }

    #[test]
    fn test_fast_int_min() {
        let a = FastInt::from(123_i64);
        let b = FastInt::from(456_i64);
        assert_eq!(a.min(b), a);
    }

    #[test]
    fn test_fast_int_to_f32() {
        let a = FastInt::from(123_i64);
        assert_eq!(a.to_f32(), 123.0_f32);
    }

    #[test]
    fn test_fast_int_to_f64() {
        let a = FastInt::from(123_i64);
        assert_eq!(a.to_f64(), 123.0_f64);
    }

    #[test]
    fn test_fast_int_display() {
        let a = FastInt::from(123_i64);
        assert_eq!(format!("{}", a), "123");
    }

    #[test]
    fn test_fast_int_checked_mul() {
        let a = FastInt::from(2_i64);
        let b = FastInt::from(3_i64);
        let result = a.checked_mul(b).unwrap();
        assert_eq!(result, FastInt::from(6_i64));
    }

    #[test]
    fn test_fast_int_mul_clamp_to_tick() {
        let a = FastInt::from(2_i64);
        let b = FastInt::from(3_i64);
        let result = a.mul_clamp_to_tick(b);
        assert_eq!(result, FastInt::from(6_i64));
    }

    #[test]
    fn test_fast_int_mul_zero_okay() {
        let a = FastInt::from(2_i64);
        let b = FastInt::from(3_i64);
        let result = a.mul_zero_okay(b);
        assert_eq!(result, FastInt::from(6_i64));
    }
}
