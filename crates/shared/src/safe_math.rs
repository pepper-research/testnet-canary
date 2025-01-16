//! Simple and easy-to-use library that allows safe operations over numericals
//! Acts as an alternative to `FastInt` and `Fractional` wherever applicable.

use crate::error::MathResult;
use crate::MathError;

macro_rules! assign_method {
    ($name:ident, $op:ident, $doc_op:expr) => {
        assign_method!($name, $op, Self, $doc_op);
    };
    ($name:ident, $op:ident, $rhs_ty:ty, $doc_op:expr) => {
        #[doc = "Safe variant of `"]
        #[doc = $doc_op]
        #[doc = "`."]
        #[inline]
        fn $name(&mut self, other: $rhs_ty) -> Result<(), MathError> {
            *self = self.$op(other)?;
            Ok(())
        }
    };
}

pub trait SafeMath<Rhs = Self>: Sized + Copy {
    const ZERO: Self;
    const ONE: Self;

    /// Safe variant of `+` that guards against overflow.
    fn safe_add(&self, other: Rhs) -> MathResult<Self>;

    /// Safe variant of `-` that guards against overflow.
    fn safe_sub(&self, other: Rhs) -> MathResult<Self>;

    /// Safe variant of `*` that guards against overflow.
    fn safe_mul(&self, other: Rhs) -> MathResult<Self>;

    /// Safe variant of `/` that guards against division by 0.
    fn safe_div(&self, other: Rhs) -> MathResult<Self>;

    /// Safe variant of `%` that guards against division by 0.
    fn safe_rem(&self, other: Rhs) -> MathResult<Self>;

    /// Safe variant of `<<` that guards against overflow.
    fn safe_shift_left(&self, other: u32) -> MathResult<Self>;

    /// Safe variant of `>>` that guards against overflow.
    fn safe_shift_right(&self, other: u32) -> MathResult<Self>;

    assign_method!(safe_add_assign, safe_add, Rhs, "+=");
    assign_method!(safe_sub_assign, safe_sub, Rhs, "-=");
    assign_method!(safe_mul_assign, safe_mul, Rhs, "*=");
    assign_method!(safe_div_assign, safe_div, Rhs, "/=");
    assign_method!(safe_rem_assign, safe_rem, Rhs, "%=");
    assign_method!(safe_shift_left_assign, safe_shift_left, u32, "<<=");
    assign_method!(safe_shift_right_assign, safe_shift_right, u32, ">>=");
}

macro_rules! impl_safe_arith {
    ($typ:ty) => {
        impl SafeMath for $typ {
            const ZERO: Self = 0;
            const ONE: Self = 1;

            #[inline]
            fn safe_add(&self, other: Self) -> Result<Self, MathError> {
                self.checked_add(other).ok_or(MathError::NumericalOverflow)
            }

            #[inline]
            fn safe_sub(&self, other: Self) -> Result<Self, MathError> {
                self.checked_sub(other).ok_or(MathError::NumericalOverflow)
            }

            #[inline]
            fn safe_mul(&self, other: Self) -> Result<Self, MathError> {
                self.checked_mul(other).ok_or(MathError::NumericalOverflow)
            }

            #[inline]
            fn safe_div(&self, other: Self) -> Result<Self, MathError> {
                self.checked_div(other).ok_or(MathError::DivisionByZero)
            }

            #[inline]
            fn safe_rem(&self, other: Self) -> Result<Self, MathError> {
                self.checked_rem(other).ok_or(MathError::DivisionByZero)
            }

            #[inline]
            fn safe_shift_left(&self, other: u32) -> Result<Self, MathError> {
                self.checked_shl(other).ok_or(MathError::NumericalOverflow)
            }

            #[inline]
            fn safe_shift_right(&self, other: u32) -> Result<Self, MathError> {
                self.checked_shr(other).ok_or(MathError::NumericalOverflow)
            }
        }
    };
}

impl_safe_arith!(u8);
impl_safe_arith!(u16);
impl_safe_arith!(u32);
impl_safe_arith!(u64);
impl_safe_arith!(usize);
impl_safe_arith!(i8);
impl_safe_arith!(i16);
impl_safe_arith!(i32);
impl_safe_arith!(i64);
impl_safe_arith!(isize);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let x = 10u32;
        let y = 11;
        assert_eq!(x.safe_add(y), Ok(x + y));
        assert_eq!(y.safe_sub(x), Ok(y - x));
        assert_eq!(x.safe_mul(y), Ok(x * y));
        assert_eq!(x.safe_div(y), Ok(x / y));
        assert_eq!(x.safe_rem(y), Ok(x % y));

        assert_eq!(x.safe_shift_left(1), Ok(x << 1));
        assert_eq!(x.safe_shift_right(1), Ok(x >> 1));
    }

    #[test]
    fn mutate() {
        let mut x = 0u8;
        x.safe_add_assign(2).unwrap();
        assert_eq!(x, 2);
        x.safe_sub_assign(1).unwrap();
        assert_eq!(x, 1);
        x.safe_shift_left_assign(1).unwrap();
        assert_eq!(x, 2);
        x.safe_mul_assign(3).unwrap();
        assert_eq!(x, 6);
        x.safe_div_assign(4).unwrap();
        assert_eq!(x, 1);
        x.safe_shift_right_assign(1).unwrap();
        assert_eq!(x, 0);
    }

    #[test]
    fn errors() {
        assert!(u32::MAX.safe_add(1).is_err());
        assert!(u32::MIN.safe_sub(1).is_err());
        assert!(u32::MAX.safe_mul(2).is_err());
        assert!(u32::MAX.safe_div(0).is_err());
        assert!(u32::MAX.safe_rem(0).is_err());
        assert!(u32::MAX.safe_shift_left(32).is_err());
        assert!(u32::MAX.safe_shift_right(32).is_err());
        assert!(u32::MAX.safe_shift_left(32).is_err());
        assert!(u32::MAX.safe_shift_right(32).is_err());
    }
}
