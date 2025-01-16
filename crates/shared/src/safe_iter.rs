use crate::{MathResult, SafeMath};

pub trait SafeArithIter<T> {
    fn safe_sum(self) -> MathResult<T>;
}

/// Iterator implementation I is an `Iterator` structure over T whose type is `SafeMath`
impl<I, T> SafeArithIter<T> for I
where
    I: Iterator<Item = T> + Sized,
    T: SafeMath,
{
    fn safe_sum(mut self) -> MathResult<T> {
        self.try_fold(T::ZERO, |x, y| x.safe_add(y))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::MathError;

    #[test]
    fn empty_sum() {
        let v: Vec<u64> = vec![];
        assert_eq!(v.into_iter().safe_sum(), Ok(0));
    }

    #[test]
    fn unsigned_sum_small() {
        let arr = [400u64, 401, 402, 403, 404, 405, 406];
        assert_eq!(
            arr.iter().copied().safe_sum().unwrap(),
            arr.iter().copied().sum::<u64>()
        );
    }

    #[test]
    fn unsigned_sum_overflow() {
        let v = vec![u64::MAX, 1];
        assert_eq!(v.into_iter().safe_sum(), Err(MathError::NumericalOverflow));
    }

    #[test]
    fn signed_sum_small() {
        let v = vec![-1i64, -2i64, -3i64, 3, 2, 1];
        assert_eq!(v.into_iter().safe_sum(), Ok(0));
    }

    #[test]
    fn signed_sum_overflow_above() {
        let v = vec![1, 2, 3, 4, i16::MAX, 0, 1, 2, 3];
        assert_eq!(v.into_iter().safe_sum(), Err(MathError::NumericalOverflow));
    }

    #[test]
    fn signed_sum_overflow_below() {
        let v = vec![i16::MIN, -1];
        assert_eq!(v.into_iter().safe_sum(), Err(MathError::NumericalOverflow));
    }

    #[test]
    fn signed_sum_almost_overflow() {
        let arr = [i64::MIN, 1, -1i64, i64::MAX, i64::MAX, 1];
        assert_eq!(
            arr.iter().copied().safe_sum().unwrap(),
            arr.iter().copied().sum::<i64>()
        );
    }
}
