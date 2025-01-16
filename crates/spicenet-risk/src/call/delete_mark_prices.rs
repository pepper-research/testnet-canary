use anyhow::Result;
use sov_modules_api::{Context, EventEmitter, Spec, TxState};

use spicenet_shared::dex::MarketProductGroup;

use crate::event::Event;
use crate::RiskModule;
use spicenet_shared::risk::RiskError;

impl<S: Spec> RiskModule<S> {
    pub(crate) fn delete_mark_prices(
        &self,
        mpg: &MarketProductGroup<S>,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        if context.sender().as_ref() != mpg.mpg_authority.as_ref() {
            return Err(RiskError::InvalidAuthority.into());
        }

        self.mark_prices.remove(&mpg.id, state)?;

        self.emit_event(state, Event::MarkPricesDeleted { mpg_id: mpg.id });

        Ok(())
    }
}
