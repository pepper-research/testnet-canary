//! Fractional is an implementation of a lightweight and simple library
//! to work float and similar structures in Rust. This library aims to provide a simple
//! and efficient alternative to "f"(32, 64, and so on) and allows creation of float objects
//! just by providing a mantissa and an exponent. The library also contains support for common mathematical
//! operations, such as addition, division, multiplication and subtraction out of the box.

use std::str::FromStr;

#[allow(dead_code)]
use {
    crate::error::MathError,
    borsh::{BorshDeserialize, BorshSerialize},
    bytemuck::{Pod, Zeroable},
    serde::{Deserialize, Serialize},
    std::{
        cmp::Ordering,
        fmt::Display,
        ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub},
    },
};

// constants
pub const DIVISION_PRECISION: i64 = 10;
pub const SQRT_PRECISION: i64 = 4; // Should always be even
pub const FLOATING_PRECISION: i64 = 10;
pub const I64_MAX: i128 = i64::MAX as i128;
pub const EXP_UPPER_LIMIT: u64 = 15;

pub fn fp32_mul(a: u64, b_fp_32: u64) -> u64 {
    (((a as u128) * (b_fp_32 as u128)) >> 32) as u64
}

// utility fn
pub fn is_num_i64(num: i128) -> bool {
    !(num > (i64::MAX as i128) || num < (i64::MIN as i128))
}

pub fn integer_sqrt(m: i128) -> std::result::Result<i128, MathError> {
    let mut start = 0_i128;
    let mut sqrt = 0_i128;

    if m < 0 {
        Err(MathError::SqrtError)
    } else if m == 0 {
        Ok(0)
    } else if m > 1 {
        let mut end = 2;

        while end * end <= m {
            end *= 2;
        }
        end += 1;

        while start <= end {
            let mid = (start + end) / 2;

            if mid * mid == m {
                sqrt = mid;
                break;
            }

            if mid * mid < m {
                sqrt = start;
                start = mid + 1
            } else {
                end = mid - 1;
            }
        }

        Ok(sqrt)
    } else {
        Ok(1)
    }
}

pub fn integer_div(m: u128, other: u128) -> std::result::Result<u128, MathError> {
    if other == 0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(m / other)
    }
}

const POW10: [i64; 19] = [
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
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
];

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[repr(C)]
#[derive(
    Debug,
    Default,
    BorshSerialize,
    BorshDeserialize,
    Clone,
    Copy,
    Pod,
    Zeroable,
    Serialize,
    Deserialize,
)]
pub struct Fractional {
    pub m: i64,
    pub exp: u64,
}

impl Display for Fractional {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base = POW10[self.exp as usize];
        if base == 0 {
            return write!(f, "0");
        }

        let lhs = self.m / base;
        let rhs = format!(
            "{:0width$}",
            (self.m % base).abs(),
            width = self.exp as usize
        );
        write!(f, "{},{}", lhs, rhs)
    }
}

pub const ZERO_FRAC: Fractional = Fractional { m: 0, exp: 0 };

pub const TWO_FRAC: Fractional = Fractional { m: 2, exp: 0 };

impl Neg for Fractional {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            m: -self.m,
            exp: self.exp,
        }
    }
}

impl Add for Fractional {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let (m, exp) = if self.exp > rhs.exp {
            (self.m + rhs.round_up(self.exp as u32).unwrap(), self.exp)
        } else if self.exp < rhs.exp {
            (self.round_up(rhs.exp as u32).unwrap() + rhs.m, rhs.exp)
        } else {
            (self.m + rhs.m, self.exp)
        };

        Self { m, exp }
    }
}

impl Fractional {
    #[must_use]
    pub fn new(m: i64, e: u64) -> Fractional {
        if e > 15 {
            // make this a constant
            panic!("exponent cannot exceed {}", 15)
        }

        Fractional { m, exp: e }
    }

    fn round_up(&self, digits: u32) -> std::result::Result<i64, MathError> {
        let diff = digits as usize - self.exp as usize;
        (self.m)
            .checked_mul(POW10[diff])
            .ok_or(MathError::NumericalOverflow)
    }

    pub fn get_reduced_form(&self) -> Self {
        let mut reduced = Fractional::new(self.m, self.exp);
        if reduced.m == 0 {
            reduced.exp = 0;
            return reduced;
        }

        while reduced.m % 10 == 0 && reduced.exp > 0 {
            reduced.m /= 10;
            reduced.exp -= 1;
        }

        reduced
    }

    pub fn reduce_from_i128_unchecked(
        mut m: i128,
        mut exp: u64,
    ) -> std::result::Result<Self, MathError> {
        if m == 0 {
            exp = 0
        }

        while (exp > 10i64 as u64) || (!is_num_i64(m) && exp > 0) {
            m /= 10;
            exp -= 1;
        }

        if !is_num_i64(m) {
            return Err(MathError::NumericalOverflow);
        }

        Ok(Fractional::new(m as i64, exp))
    }

    pub fn sign(&self) -> i32 {
        -2 * (self.is_negative() as i32) + 1
    }

    pub fn is_negative(&self) -> bool {
        self.m < 0
    }

    pub fn abs(&self) -> Fractional {
        Fractional {
            m: self.m.abs(),
            exp: self.exp,
        }
    }

    pub fn round_sf_unchecked(&self, digits: u32) -> Self {
        if digits >= self.exp as u32 {
            Fractional::new(self.m, self.exp)
        } else {
            let m = self.m / POW10[self.exp as usize - digits as usize];
            Fractional::new(m, digits as u64)
        }
    }

    pub fn to_int(&self) -> i64 {
        self.to_int_with_remainder().0
    }

    pub fn to_int_with_remainder(&self) -> (i64, Fractional) {
        let reduced_form = self.get_reduced_form();

        let int = reduced_form.m / POW10[reduced_form.exp as usize];

        (int, *self + (-int))
    }

    pub fn to_float(&self) -> f64 {
        self.to_int() as f64 + self.to_int_with_remainder().1.to_float()
    }

    pub fn from_str(s: &str) -> std::result::Result<Fractional, MathError> {
        match s.split_once(".") {
            Some((lhs, rhs)) => {
                let m = format!("{}{}", lhs, rhs)
                    .parse::<i64>()
                    .map_err(|_| MathError::DeserializationError)?;
                Ok(Fractional::new(m, rhs.len() as u64))
            }
            None => {
                let m = s
                    .parse::<i64>()
                    .map_err(|_| MathError::DeserializationError)?;
                Ok(Fractional::new(m, 0))
            }
        }
    }

    pub fn min(&self, other: Fractional) -> Fractional {
        match *self > other {
            true => other,
            false => *self,
        }
    }

    pub fn max(&self, other: Fractional) -> Fractional {
        match *self > other {
            true => *self,
            false => other,
        }
    }

    pub fn reduce_mut(&mut self) {
        if self.m == 0 {
            self.exp = 0;
            return;
        }

        while self.m % 10 == 0 {
            self.m /= 10;
            self.exp -= 1;
        }
    }

    pub fn reduce_from_i128(m: &mut i128, exp: &mut u64) -> std::result::Result<Self, MathError> {
        if *m == 0 {
            *exp = 0;
        }
        if *m % POW10[16] as i128 == 0 && *exp >= 16 {
            *m /= POW10[16] as i128;
            *exp -= 16;
        }
        if *m % POW10[8] as i128 == 0 && *exp >= 8 {
            *m /= POW10[8] as i128;
            *exp -= 8;
        }
        if *m % POW10[4] as i128 == 0 && *exp >= 4 {
            *m /= POW10[4] as i128;
            *exp -= 4;
        }
        if *m % POW10[2] as i128 == 0 && *exp >= 2 {
            *m /= POW10[2] as i128;
            *exp -= 2;
        }
        while *m % 10 == 0 && *exp > 0 {
            *m /= 10;
            *exp -= 1;
        }

        if !is_num_i64(*m) || *exp > 15 {
            return Err(MathError::NumericalOverflow);
        }

        Ok(Fractional::new(*m as i64, *exp))
    }

    pub fn reduce_unchecked(m: &mut i128, exp: &mut u64, precision: u64) -> Self {
        if *m == 0 {
            return Fractional::new(0, 0);
        }

        while *exp > precision {
            *m /= 10;
            *exp -= 1;
        }

        Fractional::new(*m as i64, *exp)
    }

    pub fn reduce(
        m: &mut i128,
        exp: &mut u64,
        precision: u64,
    ) -> std::result::Result<Self, MathError> {
        if *m == 0 {
            return Ok(Fractional::new(0, 0));
        }

        while *exp > precision {
            if *m % 10 != 0 {
                return Err(MathError::RoundError.into());
            }

            *m /= 10;
            *exp -= 1;
        }

        if !is_num_i64(*m) {
            return Err(MathError::NumericalOverflow.into());
        }

        Ok(Fractional::new(*m as i64, *exp))
    }

    pub fn round_unchecked(&self, digits: u32) -> std::result::Result<Fractional, MathError> {
        let diff = digits as i32 - self.exp as i32;
        if diff >= 0 {
            Ok(Fractional::new(
                (self.m)
                    .checked_mul(POW10[diff as usize])
                    .ok_or(MathError::NumericalOverflow)?,
                digits as u64,
            ))
        } else {
            Ok(Fractional::new(
                self.m / POW10[diff.abs() as usize],
                digits as u64,
            ))
        }
    }

    pub fn round(&self, digits: u32) -> std::result::Result<Self, MathError> {
        let num = self.round_unchecked(digits)?;
        if &num != self {
            return Err(MathError::RoundError.into());
        }
        Ok(num)
    }

    pub fn round_sf(&self, digits: u32) -> Self {
        if digits >= self.exp as u32 {
            Fractional::new(self.m, self.exp)
        } else {
            let m = self.m / POW10[self.exp as usize - digits as usize];
            Fractional::new(m, digits as u64)
        }
    }

    pub fn checked_add(
        &self,
        other: impl Into<Fractional>,
    ) -> std::result::Result<Fractional, MathError> {
        let other = other.into();
        let (mut m, mut exp) = if self.exp > other.exp {
            (
                self.m as i128 + other.round_up(self.exp as u32)? as i128,
                self.exp,
            )
        } else if self.exp < other.exp {
            (
                self.round_up(other.exp as u32)? as i128 + other.m as i128,
                other.exp,
            )
        } else {
            (self.m as i128 + other.m as i128, self.exp)
        };

        if i128::abs(m) > i64::max_value() as i128 {
            Fractional::reduce_from_i128(&mut m, &mut exp)
        } else {
            Ok(Self { m: m as i64, exp })
        }
    }

    pub fn checked_addi(
        &mut self,
        other: impl Into<Fractional>,
    ) -> std::result::Result<(), MathError> {
        *self = self.checked_add(other)?;
        Ok(())
    }

    pub fn checked_sub(
        &self,
        other: impl Into<Fractional>,
    ) -> std::result::Result<Fractional, MathError> {
        let other = other.into();
        let (mut m, mut exp) = if self.exp > other.exp {
            (
                self.m as i128 - other.round_up(self.exp as u32)? as i128,
                self.exp,
            )
        } else if self.exp < other.exp {
            (
                self.round_up(other.exp as u32)? as i128 - other.m as i128,
                other.exp,
            )
        } else {
            (self.m as i128 - other.m as i128, other.exp)
        };

        if i128::abs(m) > i64::max_value() as i128 {
            Fractional::reduce_from_i128(&mut m, &mut exp)
        } else {
            Ok(Self { m: m as i64, exp })
        }
    }

    pub fn checked_mul(
        &self,
        other: impl Into<Fractional>,
    ) -> std::result::Result<Fractional, MathError> {
        let other = other.into();
        match self.m == 0 || other.m == 0 {
            true => Ok(ZERO_FRAC),
            false => {
                let mut m = (self.m as i128) * (other.m as i128);
                let mut exp = self.exp + other.exp;
                Ok(Fractional::reduce_from_i128(&mut m, &mut exp)?)
            }
        }
    }

    pub fn saturating_mul(&self, other: impl Into<Fractional>) -> Fractional {
        match self.checked_mul(other) {
            Ok(f) => f,
            _ => Fractional::new(i64::MAX, 0),
        }
    }

    pub fn saturating_add(&self, other: impl Into<Fractional>) -> Fractional {
        match self.checked_add(other) {
            Ok(f) => f,
            _ => Fractional::new(i64::MAX, 0),
        }
    }

    /// How checked div works.
    /// 1. First, left shift (in base-10) the mantissa of the dividend
    /// 2. Second, do i128 integer division: (shifted dividend / unshifted divisor)
    /// 3. Finally, right shift back when casting the i128 result back to Fractional
    ///
    /// Further explanation:
    /// - In step 3, we call Fractional::reduce_from_i128(m, exp)?. This step could panic
    ///   when exp>EXP_UPPER_LIMIT(15). This could happen because we used to calculate the shift blindly
    ///   in step 1 and then factor in the shift in step 3.
    /// - Now, we do not do this as we cap the shift at EXP_UPPER_LIMIT-exp to prevent the function from failing
    pub fn checked_div(
        &self,
        other: impl Into<Fractional>,
    ) -> std::result::Result<Fractional, MathError> {
        let other = other.into();
        let sign = self.sign() * other.sign();
        let mut dividend: u128 = self.m.abs() as u128;
        let divisor: u128 = other.m.abs() as u128;
        let mut exp = (self.exp as i64) - (other.exp as i64);
        // in both cases, left shift (in base 10) such that exp >= other.exp + 10
        let shift = if exp >= 0 { 10 } else { 10 - exp }
            .min((15 - self.exp) as i64)
            .max(0);
        dividend = dividend
            .checked_mul(POW10[shift as usize] as u128)
            .ok_or(MathError::NumericalOverflow)?;

        let quotient: u128 = dividend / divisor;
        exp += shift;

        let divided = Fractional::reduce_from_i128(&mut (quotient as i128), &mut (exp as u64))?;
        Ok(if sign >= 0 {
            divided
        } else {
            Fractional::new(-1 * divided.m, divided.exp)
        })
    }

    pub fn sqrt(&self) -> std::result::Result<Fractional, MathError> {
        let mut exp = self.exp;
        let mut m = self.m as i128;

        if exp % 2 != 0 {
            if m < i64::MAX as i128 {
                // make i64::MAX a i128 a constant
                m *= 10;
                exp += 1;
            } else {
                m /= 10; // huge number does not matter if we lose precision!!
                exp -= 1;
            }
        }
        let mut add_exp = 2;

        for _ in 0..4 / 2 {
            // add a constant for this
            let pre_m = m * POW10[2] as i128;
            if pre_m > i64::MAX as i128 {
                // make i64::MAX a i128 a constant
                break;
            }
            m = pre_m;
            add_exp += 2;
        }

        exp += (add_exp - 2) as u64;

        let int_sqrt_m = integer_sqrt(m)?;

        if !is_num_i64(int_sqrt_m) {
            return Err(MathError::NumericalOverflow);
        }
        Ok(Fractional::new(int_sqrt_m as i64, exp / 2))
    }

    pub fn exp(&self) -> std::result::Result<Fractional, MathError> {
        let x = *self;
        let e_x = if x > Fractional::new(-1, 0) {
            Fractional::new(1, 0)
                .checked_add(x)?
                .checked_add(x * x * Fractional::new(5, 1))?
        } else if x > Fractional::new(-15, 1) {
            Fractional::new(22, 2)
        } else if x > Fractional::new(-2, 0) {
            Fractional::new(13, 2)
        } else if x > Fractional::new(-25, 1) {
            Fractional::new(8, 2)
        } else if x > Fractional::new(-3, 0) {
            Fractional::new(5, 2)
        } else {
            ZERO_FRAC
        };

        Ok(e_x)
    }

    // TODO: implement rest
}

impl AddAssign for Fractional {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs)
    }
}

impl Sub for Fractional {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let (m, exp) = if self.exp > rhs.exp {
            (self.m - rhs.round_up(self.exp as u32).unwrap(), self.exp)
        } else if self.exp < rhs.exp {
            (self.round_up(rhs.exp as u32).unwrap() - rhs.m, rhs.exp)
        } else {
            (self.m - rhs.m, self.exp)
        };

        Self { m, exp }
    }
}

impl Mul for Fractional {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let reduced_self = self.get_reduced_form();
        let reduced_rhs = rhs.get_reduced_form();

        let m = reduced_self.m as i128 * reduced_rhs.m as i128; // converted to i128 to prevent overflow
        let exp = reduced_self.exp + reduced_rhs.exp;

        match Fractional::reduce_from_i128_unchecked(m, exp) {
            Ok(v) => v,
            Err(_) => ZERO_FRAC,
        }
    }
}

impl MulAssign for Fractional {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul(rhs)
    }
}

impl Div for Fractional {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let sign = self.sign() * rhs.sign();

        let reduced_self = self.get_reduced_form();
        let reduced_rhs = rhs.get_reduced_form();

        let mut dividend: u128 = reduced_self.m.abs() as u128;
        let divisor: u128 = reduced_rhs.m.abs() as u128;
        let exp = (reduced_self.exp as i64) - (reduced_rhs.exp as i64);
        dividend *= POW10[(10 - exp.min(0)) as usize] as u128; // add a constant for this

        let quotient: u128 = dividend / divisor;

        let mut divided_val = Fractional::new(quotient as i64, (exp - exp.min(0) + 10) as u64)
            .round_sf_unchecked(10 as u32); // add a constant for this

        if sign < 0 {
            divided_val.m *= -1;
        }

        divided_val
    }
}

impl DivAssign for Fractional {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.div(rhs)
    }
}

impl PartialOrd for Fractional {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.is_negative(), other.is_negative()) {
            (false, true) => return Some(Ordering::Greater),
            (true, false) => return Some(Ordering::Less),
            _ => {} // returning None here returns weird errors
        }

        if self.m == 0 {
            return 0.partial_cmp(&other.m);
        } else if other.m == 0 {
            return self.m.partial_cmp(&0);
        }
        (self.m as i128 * POW10[other.exp as usize] as i128)
            .partial_cmp(&(other.m as i128 * POW10[self.exp as usize] as i128))
    }
}

impl PartialEq for Fractional {
    fn eq(&self, other: &Self) -> bool {
        if self.m == other.m && self.exp == other.exp {
            return true;
        }

        if self.m == 0 {
            return other.m == 0;
        } else if other.m == 0 {
            return self.m == 0;
        }

        match self.partial_cmp(other) {
            Some(Ordering::Equal) => true,
            _ => false,
        }
    }
}

impl From<u32> for Fractional {
    fn from(value: u32) -> Self {
        Fractional::new(value as i64, 0)
    }
}

impl From<u64> for Fractional {
    fn from(value: u64) -> Self {
        Fractional::new(value as i64, 0)
    }
}

impl From<usize> for Fractional {
    fn from(value: usize) -> Self {
        Fractional::new(value as i64, 0)
    }
}

impl From<i32> for Fractional {
    fn from(value: i32) -> Self {
        Fractional::new(value as i64, 0)
    }
}

impl From<i64> for Fractional {
    fn from(value: i64) -> Self {
        Fractional::new(value, 0)
    }
}

impl Mul<i64> for Fractional {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        self * Fractional::from(rhs)
    }
}

impl Add<i64> for Fractional {
    type Output = Self;

    fn add(self, rhs: i64) -> Self::Output {
        self + Fractional::from(rhs)
    }
}

impl Add<Fractional> for i64 {
    type Output = Fractional; // this is surpisingly odd -- using type Output = Self returns an error with regards to the `add` function, but using Fractional does not.

    fn add(self, rhs: Fractional) -> Self::Output {
        rhs + self
    }
}

impl Mul<Fractional> for i64 {
    type Output = Fractional; // this is surpisingly odd -- using type Output = Self returns an error with regards to the `mul` function, but using Fractional does not.

    fn mul(self, rhs: Fractional) -> Self::Output {
        Fractional::from(self) * rhs
    }
}

pub fn bps(value: i64) -> Fractional {
    Fractional::new(value, 4)
}

impl Eq for Fractional {}

impl FromStr for Fractional {
    type Err = MathError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(".") {
            Some((lhs, rhs)) => {
                let m = format!("{}{}", lhs, rhs)
                    .parse::<i64>()
                    .map_err(|_| MathError::DeserializationError)?;
                Ok(Fractional::new(m, rhs.len() as u64))
            }
            None => {
                let m = s
                    .parse::<i64>()
                    .map_err(|_| MathError::DeserializationError)?;
                Ok(Fractional::new(m, 0))
            }
        }
    }
}

// impl From<f64> for Fractional {
//     fn from(value: f64) -> Self {
//         if value == 0.0 {
//             return ZERO_FRAC;
//         }
//         let mut m = value.abs();
//         let mut exp: i32 = 0;
//         while m < 1.0 && exp > i32::MIN / 2 {
//             m *= 10.0;
//             exp -= 1;
//         }
//         while m >= 10.0 && exp < i32::MAX / 2 {
//             m /= 10.0;
//             exp += 1;
//         }
//         let sign = if value.is_sign_negative() { -1 } else { 1 };
//         Fractional::new(sign * (m as i64), exp.max(0) as u64)
//     }
// }

#[cfg(test)]
#[test]
fn test_fractional() {
    let big_int_0 = 1 << 103_i128;
    let big_int_1 = 1 << 100_i128;

    let sqrt_0 = 1 << 51_i128;
    let sqrt_1 = 1 << 50_i128;

    let sqrt_m = integer_sqrt(big_int_0 + big_int_1).unwrap_or(-1);
    assert_eq!(sqrt_m, sqrt_0 + sqrt_1);

    let big_int_0 = 1 << 126;
    let big_int_1 = 1 << 125;
    let quot = integer_div(big_int_0, big_int_1).unwrap_or(0);

    assert_eq!(quot, 2);

    // Correct rounding
    let m_round = match Fractional::new(1256000000000000, 12).round(6) {
        Ok(v) => v,
        Err(_) => ZERO_FRAC,
    };
    assert_eq!(m_round.m, 1256000000);
    assert_eq!(m_round.exp, 6);

    // Incorrect rounding
    let m_round = match Fractional::new(1, 12).round(6) {
        Ok(v) => v,
        Err(_) => Fractional::new(-1, 0),
    };
    assert_eq!(m_round.m, -1);
    assert_eq!(m_round.exp, 0);

    // More rounding
    let m_round = match Fractional::new(16375, 2).round(6) {
        Ok(v) => v,
        Err(_) => ZERO_FRAC,
    };
    assert_eq!(m_round.m, 163750000);
    assert_eq!(m_round.exp, 6);

    // reduce from i128: success
    let mut m = i64::MAX as i128;
    let mut exp = 0_u64;

    let m_frac = match Fractional::reduce_from_i128(&mut m, &mut exp) {
        Ok(v) => v,
        Err(_) => ZERO_FRAC,
    };
    let match_value = i64::MAX as i128;
    assert_eq!(m_frac.m as i128, match_value);

    // failure
    let mut m = i64::MAX as i128 + 1;
    let mut exp = 0_u64;
    let m_frac = match Fractional::reduce_from_i128(&mut m, &mut exp) {
        Ok(v) => v,
        Err(_) => ZERO_FRAC,
    };
    assert_eq!(m_frac.m as i128, 0);

    //round_sf
    let m = Fractional::new(i64::MAX, 7);

    let m_round = m.round_sf(10);
    assert_eq!(m_round.m, i64::MAX);
    assert_eq!(m_round.exp, 7);

    let m_round = m.round_sf(4);

    assert_eq!(m_round.m, i64::MAX / 10_i128.pow(3) as i64);
    assert_eq!(m_round.exp, 4);

    // Big number comparisions:
    // `big_int` 2**31 (~10**9) can only be added to dust ~10**-9
    // This is because of the shifts 2**63-1 (~10**18)
    // Increasing `big_int` or increasing precision will cause failure
    let big_int = (1 << 31) as i64;
    let num = Fractional::new(big_int, 0);
    let dust = Fractional::new(1, 9);

    let big_add = match num.checked_add(dust) {
        Ok(v) => v,
        Err(_) => ZERO_FRAC,
    };

    let big_sub = match num.checked_sub(dust) {
        Ok(v) => v,
        Err(_) => ZERO_FRAC,
    };

    assert!(big_add > num);
    assert!(big_sub < num);

    // This fails
    let big_int = (1 << 31) as i64;
    let num = Fractional::new(big_int, 0);
    let dust = Fractional::new(1, 10);

    let big_add = match num.checked_add(dust) {
        Ok(v) => v,
        Err(_) => ZERO_FRAC,
    };

    assert!(big_add == ZERO_FRAC);

    // checked_mul on large m
    let v = match Fractional::new(1 << 62, 4).checked_div(Fractional::new(1 << 34, 0)) {
        Ok(n) => n,
        Err(_) => ZERO_FRAC,
    };
    assert_eq!(v, Fractional::new(1 << 28, 4));

    // This will fail in checked_mul
    let v = match Fractional::new(1 << 40, EXP_UPPER_LIMIT)
        .checked_mul(Fractional::new(1 << 35, EXP_UPPER_LIMIT))
    {
        Ok(_) => 0,
        Err(_) => 1,
    };
    assert_eq!(v, 1);

    // This will not fail on * but will round down to the best value
    let v = if Fractional::new(1 << 40, EXP_UPPER_LIMIT) * Fractional::new(1 << 35, EXP_UPPER_LIMIT)
        > ZERO_FRAC
    {
        0
    } else {
        1
    };
    assert_eq!(v, 0);

    // Test division correctly caps precision shift.
    // Division works like this:
    // 1. First, left shift (in base-10) the mantissa of the dividend
    // 2. Second, do i128 integer division: (shifted dividened / unshifted divisor)
    // 3. Finally, right shift back when casting the i128 result back to Fractional
    //
    // In step 3, we call Fractional::reduce_from_i128( m, exp )?. This step,
    // historically, could fail when exp > EXP_UPPER_LIMIT = 15. We used to
    // blindly calculate the shift in step 1 and then factor in this shift in step 3
    // (by setting the exp in reduce_from_i128 accordingly).
    // Now, we do not calculate the shift blindly. We cap the shift at
    // EXP_UPPER_LIMIT - self.exp
    // to prevent the call to reduce_from_i128 from failing.
    // I am slightly worried that capping the shift like this could lead to large imprecision
    // when dealing with Fractionals that have high exps.

    // This verifies the shift is capped so the resulting exp <= EXP_UPPER_LIMIT = 15.
    // self.exp = 6, other.exp = 0, and, therefore, shift = min(10, 15 - 6 = 9) = 9.
    let r = Fractional {
        m: 26666666,
        exp: 6,
    }
    .checked_div(Fractional { m: 3, exp: 0 })
    .unwrap();
    assert_eq!(
        r,
        Fractional {
            m: 8888888666666666,
            exp: 15
        }
    );

    // This verifies negative exp diff is handled correctly.
    // self.exp = 0, other.exp = 6, and, therefore, shift = min(10 - (-6) = 16, 15-0 = 15) = 15.
    let r = Fractional { m: 3, exp: 0 }
        .checked_div(Fractional {
            m: 26666666,
            exp: 6,
        })
        .unwrap();
    assert_eq!(
        r,
        Fractional {
            m: 112500002,
            exp: 9
        }
    );

    // This verifies exp = (self.exp - other.exp) < 0 together with DIVISION_PRECISION - exp < 0 is handled correctly.
    // self.exp = 0, other.exp = 11, and, therefore, shift = max(0, min(10 - (11 - 0) = -1, 15-0 = 15)) = max(0, -1) = 0.
    let r = Fractional { m: 3, exp: 0 }
        .checked_div(Fractional {
            m: 2666666666666,
            exp: 11,
        })
        .unwrap();
    assert_eq!(r, Fractional { m: 1125, exp: 4 });

    // check a smol dividend
    let r = Fractional {
        m: 26666666,
        exp: 15,
    }
    .checked_div(Fractional { m: 3, exp: 0 })
    .unwrap();
    assert_eq!(
        r,
        Fractional {
            m: 8888888,
            exp: 15
        }
    );

    // check a smoler dividend - we can't even error handle this
    std::panic::catch_unwind(|| {
        Fractional {
            m: 26666666,
            exp: 16,
        }
        .checked_div(Fractional { m: 1, exp: 0 })
        .expect_err("Surely we can't divide by 1!")
    })
    .expect_err("Surely we can't divide by 1!");
    // assert_eq!(r, Fractional{ m: 26666666, exp: 16});

    // and a smol divisor
    let r = Fractional { m: 3, exp: 0 }
        .checked_div(Fractional {
            m: 26666666,
            exp: 15,
        })
        .unwrap();
    assert_eq!(
        r,
        Fractional {
            m: 112500002,
            exp: 0
        }
    );
}
