use anyhow::Result;
use sov_modules_api::{Context, EventEmitter, Spec, TxState};

use spicenet_shared::addresses::TrgId;

use crate::event::Event;
use crate::RiskModule;

impl<S: Spec> RiskModule<S> {
    pub fn delete_variance_cache(
        &self,
        trg_id: TrgId<S>,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        self.variance_caches.remove(&trg_id, state)?;

        self.emit_event(state, Event::VarianceCacheDeleted { trg_id });

        Ok(())
    }
}
