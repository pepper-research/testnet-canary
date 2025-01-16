use sov_modules_api::Spec;

use crate::time::Slot;
use crate::RiskError;
use crate::{FastInt, ProductId};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct CovarianceMetadata {
    // tag: RiskAccountTag, // since these are not Solana PDAs, we shouldn't need this
    pub update_slot: Slot,
    // pub authority: Address<S>,
    pub num_active_products: usize,
    // pub(crate) product_keys: [ProductId; MAX_OUTRIGHTS],
    // pub(crate) standard_deviations: [FastInt; MAX_OUTRIGHTS],
    pub product_keys: Vec<ProductId>,
    pub standard_deviations: Vec<FastInt>,
}

impl CovarianceMetadata {
    pub fn get_product_index(&self, product_key: &ProductId) -> Result<usize, RiskError> {
        for index in 0..self.num_active_products {
            if self.product_keys[index] == *product_key {
                return Ok(index);
            }
        }

        Err(RiskError::MissingCovarianceEntry.into())
    }

    pub fn get_std(&self, product_key: &ProductId) -> Result<FastInt, RiskError> {
        let index = self.get_product_index(product_key)?;

        Ok(self.standard_deviations[index])
    }
}
