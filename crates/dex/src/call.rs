use anyhow::{anyhow, Result};
use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use sov_modules_api::{CallResponse, Spec};

use crate::state::*;
use crate::Dex;
use spicenet_shared::TrgId;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    schemars(bound = "S::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub enum CallMessage<S: Spec> {
    InitTrg {},
    UpdateProductFunding {
        amount: u64,
        new_product_status: ProductStatus,
    }, // New variant for funding
}
impl<S: Spec> Dex<S> {
    pub(crate) fn initialize_trg(&self) -> Result<CallResponse> {
        Ok(CallResponse::default())
    }

    pub(crate) fn update_product_funding(
        &self,
        amount: u64,
        new_product_status: ProductStatus,
        market_product_group: MarketProductGroup<S>,
        product_id: &ProductId,
    ) -> Result<CallResponse> {
        assert!(
            market_product_group.is_initialized(),
            UtilError::AccountUninitialized
        )?;
        assert!(
            !market_product_group.is_killed,
            DexError::MarketProductGroupKillswitchIsOn
        )?;
        assert!(
            new_product_status != ProductStatus::Uninitialized,
            DexError::InvalidProductStatusInUpdateFunding
        )?;

        let (idx, _) = market_product_group.find_product_index(product_id)?;
        let cash_decimals = market_product_group.decimals;
        let product = market_product_group.active_products[idx].try_to_outright_mut()?;
        product.apply_new_funding(amount, cash_decimals)?;

        product.product_status = new_product_status;
        market_product_group.sequence_number += 1;
        Ok(CallResponse::default())
    }
}
