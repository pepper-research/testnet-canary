use anyhow::Result;
use sov_modules_api::{Context, EventEmitter, Spec, TxState};

use crate::event::Event;
use spicenet_shared::dex::MarketProductGroup;
use spicenet_shared::risk::RiskError;

use crate::RiskModule;

impl<S: Spec> RiskModule<S> {
    pub(crate) fn collect_mark_prices_garbage(
        &self,
        mpg: &MarketProductGroup<S>,
        max_products_to_examine: u8,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        // let mpg = self
        //     .dex
        //     .get_mpg(state, &mpg_id)
        //     .or_else(|_| Err(DexError::MarketProductGroupDoesNotExist.into()))?;

        let mut mark_prices = match self.mark_prices.get(&mpg.id, state).unwrap() {
            Some(mark_prices) => mark_prices,
            None => return Err(RiskError::MarkPricesNotInitialized.into()),
        };

        mark_prices.collect_garbage(&mpg, max_products_to_examine as usize)?;

        self.emit_event(
            state,
            Event::MarkPricesGarbageCollected {
                mpg_id: mpg.id,
                max_products_to_examine,
            },
        );

        Ok(())
    }
}
