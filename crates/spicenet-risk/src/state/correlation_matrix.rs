use std::ops::Div;

use spicenet_shared::{FastInt, RiskError};

use super::{CORRELATION_INDEX_LOOKUP_TABLE, CORRELATION_LOOKUP_TABLE};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct CorrelationMatrix {
    // tag: RiskAccountTag, // since these are not Solana PDAs, we shouldn't need this
    pub num_active_products: usize,
    // pub(crate) possible_correlations: [i8; MAX_CORRELATION_SIZE],
    pub possible_correlations: Vec<i8>,
}

impl CorrelationMatrix {
    // TODO: document
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
        let index = self.get_array_index(x, y)?;
        self.possible_correlations[index] = CorrelationMatrix::to_correlation_ticks(value);
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
