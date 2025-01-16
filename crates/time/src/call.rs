use anyhow::Result;
use sov_modules_api::{Context, EventEmitter, Spec, TxState};
use thiserror::Error;

use crate::constants::SLOT_TIME;
use crate::event::Event;
use crate::TimeModule;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "CallMessage")
)]
#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone,
    Eq,
)]
/// Transactions handled by the Time module, currently only update.
pub enum CallMessage {
    /// Update the slot
    UpdateSlot {},
    /// Update the timestamp
    UpdateTimestamp {},
}

#[derive(Debug, Error)]
enum UpdateTimeError {
    #[error("Only the sequencer authority can update the time state")]
    Unauthorized,
}

impl<S> TimeModule<S>
where
    S: Spec,
{
    pub fn update_slot(
        &self,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        let admin = self.admin.get_or_err(state)??;

        if admin.as_ref() != context.sender().as_ref() {
            return Err(UpdateTimeError::Unauthorized.into());
        }

        let current_slot = self.slot.get(state)?;

        let new_slot = current_slot.unwrap() + 1;

        self.slot.set(&new_slot, state)?;

        let event = Event::UpdateSlot { slot: new_slot };

        self.emit_event(state, event);

        Ok(())
    }

    pub fn update_timestamp(
        &self,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        let admin = self.admin.get_or_err(state)??;

        if admin.as_ref() != context.sender().as_ref() {
            return Err(UpdateTimeError::Unauthorized.into());
        }

        let current_unix_timestamp = self.unix_timestamp.get(state)?;

        let new_unix_timestamp = current_unix_timestamp.unwrap() + SLOT_TIME;

        self.unix_timestamp.set(&new_unix_timestamp, state)?;

        let event = Event::UpdateTime {
            unix_timestamp: new_unix_timestamp,
        };

        self.emit_event(state, event);

        Ok(())
    }
}
