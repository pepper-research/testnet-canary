use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

use crate::MathError;

/// Represents a pair of bits[0s and 1s]
/// Values are stored in the `inner` field of the struct, as a pair of two bits.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, Deserialize, Serialize, Pod,
)]
#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[repr(C)]
pub struct BitPair {
    pub inner: [u128; 2],
}

unsafe impl Zeroable for BitPair {}

impl BitPair {
    /// Finds an index in the bit array and inserts bits by performing the `bitxor` operation.
    /// XOR is a bitwise operation that takes two binary operands and conducts a logical exclusive disjunction.
    /// The result of a XOR operation is 1 if and only if the operands in context are different.
    /// Example table
    /// OP 1              OP 2               Result
    /// 1                 0                  1
    /// 0                 0                  0
    /// 0                 1                  1
    /// 1                 1                  0
    #[inline]
    pub fn find_idx_and_insert(&mut self) -> Result<usize, MathError> {
        let idx = if self.inner[0] != u128::MAX {
            (u128::MAX ^ self.inner[0]).trailing_zeros()
        } else if self.inner[1] == u128::MAX {
            return Err(MathError::InvalidBitPairIndex.into());
        } else {
            (u128::MAX ^ self.inner[1]).trailing_zeros() + 128
        } as usize;
        self.insert(idx).map(|_| idx)
    }

    /// Performs the bitwise OR operation and assigns the value back to the left-hand operand, aka x here.
    /// Example
    /// A(in bits) = 1010
    /// B(in bits) = 1100
    ///
    /// We perform the bitwise OR operation by doing A |= B. This returns the value 1110 and assigns this value back to A.
    #[inline]
    pub fn insert(&mut self, x: usize) -> std::result::Result<(), MathError> {
        if x > 255 {
            return Err(MathError::InvalidBitPairIndex.into());
        }

        self.inner[get_idx(x)] |= mask(x, get_idx(x));
        Ok(())
    }

    /// Performs the bitwise AND operation and assigns the value back to the left-hand operand, aka x here.
    /// Example
    /// A(in bits) = 1010
    /// B(in bits) = 1100
    ///
    /// We perform the bitwise OR operation by doing A &= B. This returns the value 1000 and assigns this value back to A.
    #[inline]
    pub fn remove(&mut self, x: usize) -> std::result::Result<(), MathError> {
        if x > 255 {
            return Err(MathError::InvalidBitPairIndex.into());
        }
        self.inner[get_idx(x)] &= !mask(x, get_idx(x));

        Ok(())
    }

    pub fn contains(&self, x: usize) -> bool {
        if x > 255 {
            return false;
        }

        (self.inner[get_idx(x)] & mask(x, get_idx(x))) != 0
    }
}

/// Returns the index of an element in the bitpair.
/// If x > 127, the function returns true, aka 1 or false, aka 0.
#[inline]
fn get_idx(x: usize) -> usize {
    (x > 127) as usize
}

/// Performs the "left-shift" operation on a set of bits.
/// The "<<" operation shifts the bits of the left operand to the left by the number of positions specified by the right operand.
#[inline]
fn mask(x: usize, idx: usize) -> u128 {
    1 << (x - idx * 128)
}

impl Default for BitPair {
    fn default() -> Self {
        Self { inner: [0, 0] }
    }
}
