use spicenet_shared::risk::{CovarianceMetadata, MAX_OUTRIGHTS};
use spicenet_shared::{FastInt, ProductId, RiskError};

use super::CorrelationMatrix;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct CovarianceMatrix {
    pub covariance_metadata: CovarianceMetadata,
    pub correlations: CorrelationMatrix,
    // pub(crate) mappings: [u16; MAX_PRODUCTS],
    pub mappings: Vec<u16>,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct MutableCovarianceMatrix {
    pub covariance_metadata: CovarianceMetadata,
    pub correlations: CorrelationMatrix,
}

impl CovarianceMatrix {
    pub fn new(
        metadata: CovarianceMetadata,
        correlations: CorrelationMatrix,
        // mappings: [u16; MAX_PRODUCTS],
        mappings: Vec<u16>,
    ) -> Self {
        Self {
            covariance_metadata: metadata,
            correlations,
            mappings,
        }
    }

    pub fn get_covariance(
        &self,
        pubkey_1: &ProductId,
        pubkey_2: &ProductId,
    ) -> Result<FastInt, RiskError> {
        if *pubkey_1 == *pubkey_2 {
            let product_index = self.covariance_metadata.get_product_index(pubkey_1)?;
            return self.get_covariance_from_product_indexes(product_index, product_index);
        }

        self.get_covariance_from_product_indexes(
            self.covariance_metadata.get_product_index(pubkey_1)?,
            self.covariance_metadata.get_product_index(pubkey_2)?,
        )
    }

    pub fn get_covariance_from_product_indexes(
        &self,
        product_index_1: usize,
        product_index_2: usize,
    ) -> Result<FastInt, RiskError> {
        if product_index_1 == product_index_2 {
            let std = self.covariance_metadata.standard_deviations
                [self.mappings[product_index_1] as usize];
            Ok(std * std)
        } else {
            let mapped_index_1 = self.mappings[product_index_1] as usize;
            let mapped_index_2 = self.mappings[product_index_2] as usize;

            let (lesser, greater) = match mapped_index_1 < mapped_index_2 {
                true => (mapped_index_1, mapped_index_2),
                false => (mapped_index_2, mapped_index_1),
            };

            let correlation = self.correlations.get_corr_unchecked(lesser, greater);
            Ok(self.covariance_metadata.standard_deviations[mapped_index_1]
                * self.covariance_metadata.standard_deviations[mapped_index_2]
                * correlation)
        }
    }
}

impl MutableCovarianceMatrix {
    // pub fn to_covariance_matrix(&mut self, mappings: [u16; MAX_PRODUCTS]) -> CovarianceMatrix {
    pub fn to_covariance_matrix(&mut self, mappings: Vec<u16>) -> CovarianceMatrix {
        CovarianceMatrix::new(
            self.covariance_metadata.clone(),
            self.correlations.clone(),
            mappings,
        )
    }

    pub fn set_covariance(
        &mut self,
        product_keys: &Vec<ProductId>,
        std: &Vec<FastInt>,
        correlations: &Vec<Vec<FastInt>>,
    ) -> Result<(CovarianceMetadata, CorrelationMatrix), RiskError> {
        let len = product_keys.len();

        if len > MAX_OUTRIGHTS {
            return Err(RiskError::InvalidCovarianceInput.into());
        }

        if len != std.len() {
            return Err(RiskError::InvalidCovarianceInput.into());
        }

        if len != correlations.len() {
            return Err(RiskError::InvalidCovarianceInput.into());
        }

        for correlation in correlations.iter() {
            if len != correlation.len() {
                return Err(RiskError::InvalidCovarianceInput.into());
            }
        }

        self.covariance_metadata.num_active_products = len;

        for (idx, product_key) in product_keys.iter().enumerate() {
            self.covariance_metadata.product_keys[idx] = product_key.clone()
        }

        for (idx, std_dev) in std.iter().enumerate() {
            self.covariance_metadata.standard_deviations[idx] = (*std_dev).into()
        }

        self.correlations.num_active_products = len;
        for (index_i, correlation_row) in correlations.iter().enumerate() {
            for (index_j, correlation) in correlation_row.iter().enumerate().skip(index_i) {
                self.correlations
                    .set_corr(index_i, index_j, correlation.to_f32())?;
            }
        }

        Ok((self.covariance_metadata.clone(), self.correlations.clone()))
    }
}
