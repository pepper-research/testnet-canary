use anyhow::{Error, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use sov_modules_api::{Context, Spec, TxState};
use spicenet_aaob::Slab;

use crate::RiskModule;
use spicenet_shared::{FastInt, MPGId, MarketProductGroup, OutrightProduct, ProductId, RiskError};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(Debug, Clone, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct ProductMarkPriceUpdate {
    product_id: ProductId,
    asks: Slab,
    bids: Slab,
}

impl<S: Spec> RiskModule<S> {
    // TODO(!aaob, !oracle): dependent on orderbook logic being complete for getting asks and bids and oracle being done to get oracle price
    pub(crate) fn update_mark_prices(
        &self,
        mpg: &MarketProductGroup<S>,
        products_to_update: Vec<ProductMarkPriceUpdate>,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        let mut mark_prices_array = self
            .mark_prices
            .get(&mpg.id, state)
            .unwrap()
            .ok_or::<Error>(RiskError::MarkPricesNotInitialized.into())?;

        for product in products_to_update {
            let (product_idx, outright) = mpg.find_outright(&product.product_id)?;

            let index_price = self
                .lut
                .get_price(outright.metadata.price_index, state)?
                .price;

            mark_prices_array.update_outright_price_with_slab(
                outright,
                index_price.into(),
                product_idx,
                self.time_module.get_slot(state)?.slot,
                // TODO: change to reference to avoid cloning
                product.bids.clone(),
                product.asks.clone(),
            )?;
        }

        Ok(())
    }
}
