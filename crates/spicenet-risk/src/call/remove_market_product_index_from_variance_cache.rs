use anyhow::Result;
use sov_modules_api::prelude::anyhow;
use sov_modules_api::{Context, EventEmitter, Spec, TxState};
use spicenet_shared::dex::{MarketProductGroup, TraderRiskGroup};

use spicenet_shared::ZERO_FAST_INT;

use crate::event::Event;
use crate::RiskModule;
use spicenet_shared::risk::RiskError;

impl<S: Spec> RiskModule<S> {
    pub(crate) fn remove_market_product_index_from_variance_cache(
        &self,
        mpg: &MarketProductGroup<S>,
        trg: &TraderRiskGroup<S>,
        market_product_index: usize,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        // let mpg = self
        //     .dex
        //     .get_mpg(state, &mpg_id)
        //     .or_else(|_| Err(DexError::MarketProductGroupDoesNotExist.into()))?;

        if context.sender().as_ref() != mpg.mpg_authority.as_ref() {
            return Err(RiskError::InvalidAuthority.into());
        }

        match self.covariance_matrix.get(&mpg.id, state).unwrap() {
            Some(covariance_matrix) => covariance_matrix,
            None => return Err(RiskError::CovarianceMatrixNotInitialized.into()),
        };

        let mut variance_cache = match self.variance_caches.get(&trg.id, state).unwrap() {
            Some(variance_cache) => variance_cache,
            None => return Err(RiskError::VarianceCacheNotInitialized.into()),
        };

        let mut some_variance_cache_index: Option<usize> = None;
        for (variance_cache_index, a_market_product_index) in
            variance_cache.product_indexes.iter().enumerate()
        {
            if *a_market_product_index == market_product_index {
                some_variance_cache_index = Some(variance_cache_index);
                break;
            }
        }

        match some_variance_cache_index {
            Some(variance_cache_index) => {
                // msg!("Found market product index {} at variance cache index {}. Clearing this entry of variance cache and forcing rebuild.", market_product_index, variance_cache_index);
                variance_cache.product_indexes[variance_cache_index] = usize::MAX;
                variance_cache.positions[variance_cache_index] = ZERO_FAST_INT;
                variance_cache.sigma_position[variance_cache_index] = ZERO_FAST_INT;
                variance_cache.update_offset = 0;
            }
            None => return Err(RiskError::FailedToFindMarketProductIndexInVarianceCache.into()),
        };

        self.emit_event(
            state,
            Event::MarketProductIndexRemovedFromVarianceCache {
                mpg_id: mpg.id,
                trg_id: trg.id.clone(),
                market_product_index,
            },
        );

        Ok(())
    }
}
