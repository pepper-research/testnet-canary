use anyhow::Result;
use sov_modules_api::{Context, EventEmitter, Spec, TxState};

use spicenet_shared::dex::TraderRiskGroup;
use spicenet_shared::FastInt;

use crate::event::Event;
use crate::RiskModule;
use spicenet_shared::risk::{RiskError, VarianceCache};

impl<S: Spec> RiskModule<S> {
    pub fn initialize_variance_cache(
        &self,
        trg: TraderRiskGroup<S>,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        // self.dex
        //     .get_trg(state, &trg_id)
        //     .or_else(|_| Err(DexError::TraderRiskGroupDoesNotExist.into()))?;

        match self.variance_caches.get(&trg.id, state).unwrap() {
            Some(_) => return Err(RiskError::VarianceCacheAlreadyInitialized.into()),
            None => (),
        }

        self.variance_caches
            .set(
                &trg.id,
                &VarianceCache {
                    update_offset: 0,
                    derivative_position_value: FastInt { value: 0 },
                    open_order_variance: FastInt { value: 0 },
                    total_liquidity_buffer: FastInt { value: 0 },
                    total_variance_traded: FastInt { value: 0 },
                    positions: Vec::new(),
                    product_indexes: Vec::new(),
                    sigma_position: Vec::new(),
                },
                state,
            )
            .unwrap();

        self.emit_event(state, Event::VarianceCacheInitialized { trg_id: trg.id });

        Ok(())
    }
}
