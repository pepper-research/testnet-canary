use anyhow::Result;
use sov_modules_api::Spec;

use spicenet_shared::dex::{MarketProductGroup, TraderRiskGroup};
use spicenet_shared::{IsInitialized, ProductId, RiskError};

use crate::RiskModule;
use spicenet_shared::{CovarianceMetadata, VarianceCache};

impl<S: Spec> RiskModule<S> {
    #[inline(always)]
    pub fn find_product_id(
        find_product_id: &ProductId,
        product_id_list: Vec<ProductId>,
        matched_products: Vec<bool>,
        num_active: usize,
    ) -> Option<usize> {
        for (index, product_id) in product_id_list.iter().enumerate().take(num_active) {
            if !matched_products[index] {
                if *product_id == *find_product_id {
                    return Some(index);
                }
            }
        }

        return None;
    }

    pub fn __map_traded_indexes(
        metadata: &CovarianceMetadata,
        trader_risk_group: &TraderRiskGroup<S>,
        mappings: &mut Vec<u16>,
        matched_products: &mut Vec<bool>,
    ) -> Result<(), RiskError> {
        for trader_position in trader_risk_group.trader_positions.iter() {
            if !trader_position.is_initialized() {
                continue;
            }

            match Self::find_product_id(
                &trader_position.product_key,
                metadata.product_keys.clone(), // Use clone to avoid moving out of the reference
                matched_products.to_vec(),
                metadata.num_active_products,
            ) {
                Some(index) => {
                    mappings[trader_position.product_index] = index as u16;
                    matched_products[index] = true;
                }
                None => {
                    // msg!(
                    //     "Missing covariance for traded product {}",
                    //     trader_position.product_key
                    // );
                    return Err(RiskError::MissingCovarianceEntry.into());
                }
            }
        }

        Ok(())
    }

    pub fn __map_variance_indexes(
        metadata: &CovarianceMetadata,
        market_product_group: &MarketProductGroup<S>,
        variance_cache: &VarianceCache,
        mappings: &mut Vec<u16>,
        matched_products: &mut Vec<bool>,
    ) -> Result<(), RiskError> {
        // map the variance products that could have been removed
        for product_index in variance_cache.product_indexes.iter() {
            // if it's an active product in the VarianceCache
            if *product_index == usize::MAX {
                continue;
            }
            // if the product hasn't already been mapped
            if mappings[*product_index] < u16::MAX {
                continue;
            }

            let product_id = &market_product_group.active_products.array[*product_index]
                .try_to_outright()
                .unwrap()
                .metadata
                .product_id;

            match Self::find_product_id(
                product_id,
                metadata.product_keys.clone(),
                matched_products.to_vec(),
                metadata.num_active_products,
            ) {
                Some(index) => {
                    mappings[*product_index] = index as u16;
                    matched_products[index] = true;
                }
                None => {
                    // msg!(
                    //     "Missing covariance for previously traded product {}",
                    //     product_key
                    // );
                    return Err(RiskError::MissingCovarianceEntry.into());
                }
            }
        }

        Ok(())
    }

    pub fn map_traded_indexes(
        metadata: &CovarianceMetadata,
        trader_risk_group: &TraderRiskGroup<S>,
    ) -> Result<Vec<u16>> {
        let mut mappings = Vec::new();
        let mut matched_products = Vec::new();

        Self::__map_traded_indexes(
            metadata,
            trader_risk_group,
            &mut mappings,
            &mut matched_products,
        )?;

        Ok(mappings)
    }

    pub fn map_all_indexes(
        metadata: &CovarianceMetadata,
        market_product_group: &MarketProductGroup<S>,
        trader_risk_group: &TraderRiskGroup<S>,
        variance_cache: &VarianceCache,
    ) -> Result<Vec<u16>> {
        let mut mappings = Vec::new();
        let mut matched_products = Vec::new();

        Self::__map_traded_indexes(
            metadata,
            trader_risk_group,
            &mut mappings,
            &mut matched_products,
        )?;
        Self::__map_variance_indexes(
            metadata,
            market_product_group,
            variance_cache,
            &mut mappings,
            &mut matched_products,
        )?;
        Ok(mappings)
    }
}
