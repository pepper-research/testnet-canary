use sov_modules_api::{SlotHooks, Spec, StateCheckpoint};
use sov_state::Storage;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::TimeModule;

impl<S: Spec> SlotHooks for TimeModule<S> {
    type Spec = S;

    fn begin_slot_hook(
        &self,
        _visible_hash: &<<Self::Spec as Spec>::Storage as Storage>::Root,
        state: &mut StateCheckpoint<<Self::Spec as Spec>::Storage>,
    ) {
        let start = SystemTime::now();
        let timestamp = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        tracing::debug!("begin_slot_hook");
        self.unix_timestamp
            .set(&(timestamp.as_millis() as u64), state)
            .unwrap();
    }
}
