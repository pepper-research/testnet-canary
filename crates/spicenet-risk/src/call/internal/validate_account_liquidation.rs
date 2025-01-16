use anyhow::Result;
use sov_modules_api::{Context, EventEmitter, Spec, TxState};
use spicenet_shared::{RiskError, ZERO_FAST_INT};

use crate::event::Event;
use crate::RiskModule;
use spicenet_shared::dex::{MarketProductGroup, TraderRiskGroup};
use spicenet_shared::risk::HealthOutput;
use spicenet_shared::risk::RiskInfo;

impl<S: Spec> RiskModule<S> {
    pub fn validate_account_liquidation(
        &self,
        mpg: &mut MarketProductGroup<S>,
        trg: TraderRiskGroup<S>,
        params: RiskInfo,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        // let mpg = self
        //     .dex
        //     .get_mpg(state, &mpg_id)
        //     .or_else(|_| Err(DexError::MarketProductGroupDoesNotExist.into()))?;

        // let trg = self
        //     .dex
        //     .get_trg(state, &trg_id)
        //     .or_else(|_| Err(DexError::TraderRiskGroupDoesNotExist.into()))?;

        let mut covariance_matrix =
            self.covariance_matrix
                .get(&mpg.id, state)
                .unwrap()
                .ok_or::<anyhow::Error>(RiskError::CovarianceMatrixNotInitialized.into())?;
        let mark_prices = self
            .mark_prices
            .get(&mpg.id, state)
            .unwrap()
            .ok_or::<anyhow::Error>(RiskError::MarkPricesNotInitialized.into())?;

        let metadata = &covariance_matrix.covariance_metadata;
        let mut variance_cache = self
            .variance_caches
            .get(&trg.id, state)
            .unwrap()
            .ok_or::<anyhow::Error>(RiskError::VarianceCacheNotInitialized.into())?;

        for variance_cache_index in 0..variance_cache.product_indexes.len() {
            variance_cache.product_indexes[variance_cache_index] = usize::MAX;
            variance_cache.positions[variance_cache_index] = ZERO_FAST_INT;
            variance_cache.sigma_position[variance_cache_index] = ZERO_FAST_INT;
        }

        covariance_matrix.mappings =
            Self::map_all_indexes(&metadata, &mpg, &trg, &variance_cache).unwrap();

        let risk_profile = Self::calculate_risk_profile_cached(
            &mpg,
            &mark_prices,
            &trg,
            &covariance_matrix,
            &mut variance_cache,
            self.time_module.get_slot(state).unwrap().slot,
        )?;

        let liquidation_status = Self::calculate_liquidation_status(
            &risk_profile,
            &trg,
            risk_profile.get_health_status(),
        )?;

        let health_output = HealthOutput::Liquidatable { liquidation_status };

        mpg.risk_output_register.health_output = health_output.clone();

        self.emit_event(
            state,
            Event::AccountLiquidationValidation {
                mpg_id: mpg.id,
                trg_id: trg.id,
                risk_info_params: params,
                health_output,
            },
        );

        Ok(())
    }
}
