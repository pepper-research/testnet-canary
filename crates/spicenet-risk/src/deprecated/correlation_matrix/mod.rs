//! Implementation of a correlation matrix used to determine the level of correlation between different products
//! Correlation can be used in various ways. For example, highly correlated products can be combined together
//! to create a combo product. SPANDEX also allows for variable initial margin relief between two correlated products,
//! but with opposite risk, like long SOL and short mSOL. The extent of margin relief depends on the correlation between
//! the products.

use std::{mem::size_of, ops::Div};

use spicenet_shared::fast_int::FastInt;

use crate::{
    correlation_matrix::{
        correlation_index_lookup_table::CORRELATION_INDEX_LOOKUP_TABLE,
        correlation_lookup_table::CORRELATION_LOOKUP_TABLE,
    },
    error::RiskError,
    // fast_int::FastInt,
    utils::RiskStateTag,
};

pub mod correlation_index_lookup_table;
pub mod correlation_lookup_table;

// correlation matrix notes
// we represent the correlation matrix using only the elements on the main
// diagonal and those above it, like so (the values in the cells represent
// the array indexes in the CorrelationMatrix account data)

// product x0  x1  x2  x3  x4
//   y0     0   1   2   3   4
//   y1         5   6   7   8
//   y2             9  10  11
//   y3                12  13
//   y4                    14

/// Maximum allowed correlation between products
pub const MAX_CORRELATIONS: usize = 8256;

pub struct CorrelationMatrix {
    /// State verifier for variance cache.
    /// [`RiskStateTag::VarianceCache]
    pub state_identifier: RiskStateTag,

    /// The number of active products(not settled/expired) products
    pub num_active_products: usize,

    /// The maximum number of possible correlations
    pub possible_correlations: [i8; MAX_CORRELATIONS], // correlation can be negative
}

impl CorrelationMatrix {
    pub const LEN: usize = size_of::<CorrelationMatrix>();
    pub const CORRELATION_TICK: f32 = 1_f32 / 128_f32;

    #[inline(always)]
    pub fn array_size(num: usize) -> usize {
        num * (num + 1) / 2
    }

    #[inline(always)]
    pub fn get_array_index_unchecked(lesser: usize, greater: usize) -> usize {
        CORRELATION_INDEX_LOOKUP_TABLE[lesser][greater] as usize
    }

    #[inline(always)]
    pub fn get_array_index(&self, x: usize, y: usize) -> Result<usize, RiskError> {
        if x < self.num_active_products && y < self.num_active_products {
            let (lesser, greater) = match x < y {
                true => (x, y),
                false => (y, x),
            };

            let correlation_index = CORRELATION_INDEX_LOOKUP_TABLE[lesser][greater];
            if correlation_index < 0 {
                Err(RiskError::InvalidCovarianceMatrixAccess.into())
            } else {
                Ok(correlation_index as usize)
            }
        } else {
            Err(RiskError::InvalidCovarianceMatrixAccess.into())
        }
    }

    /// This function converts clamps the value given to range of `[-1, 1]`
    /// and divides it with tick value of `1/128` before rounding it down
    /// to store inside the possible_correlations array
    #[inline(always)]
    pub fn to_correlation_ticks(value: f32) -> i8 {
        value
            .max(-1_f32)
            .min(1_f32)
            .div(CorrelationMatrix::CORRELATION_TICK)
            .round() as i8
    }

    pub fn from_corr_ticks(value: i8) -> FastInt {
        CORRELATION_LOOKUP_TABLE[(value as i32 + 128_i32) as usize]
    }

    #[inline(always)]
    pub fn set_corr(&mut self, x: usize, y: usize, value: f32) -> Result<(), RiskError> {
        self.possible_correlations[self.get_array_index(x, y)?] =
            CorrelationMatrix::to_correlation_ticks(value);
        Ok(())
    }

    #[inline(always)]
    pub fn get_corr(&self, x: usize, y: usize) -> Result<FastInt, RiskError> {
        Ok(CorrelationMatrix::from_corr_ticks(
            self.possible_correlations[self.get_array_index(x, y)?],
        ))
    }

    pub fn get_corr_unchecked(&self, smaller: usize, larger: usize) -> FastInt {
        CorrelationMatrix::from_corr_ticks(
            self.possible_correlations
                [CorrelationMatrix::get_array_index_unchecked(smaller, larger)],
        )
    }
}
