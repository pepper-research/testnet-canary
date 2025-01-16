use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sov_modules_api::{
    Address, Context, GenesisState, Module, ModuleError, ModuleId, ModuleInfo, Spec,
    StateValue, TxState, DaSpec
};

pub use call::CallMessage;
pub use event::Event;

pub type Slot = u64;

pub mod call;
pub mod constants;
pub mod event;
pub mod rpc;
pub mod hooks;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "Address<S>: Serialize + serde::de::DeserializeOwned")]
pub struct TimeConfig<S: Spec> {
    pub sequencer_authority: Address<S>,
}

#[derive(Clone, ModuleInfo, sov_modules_api::ModuleRestApi)]
pub struct TimeModule<S: Spec> {
    #[id]
    /// The ID of the time module.
    id: ModuleId,

    #[state]
    slot: StateValue<Slot>,
    #[state]
    unix_timestamp: StateValue<u64>,
    #[state]
    admin: StateValue<Address<S>>,
}

impl<S: Spec> Module for TimeModule<S> {
    type Spec = S;
    type Config = TimeConfig<S>;
    type CallMessage = CallMessage;
    type Event = Event;

    fn genesis(
        &self,
        _genesis_rollup_header: &<<S as Spec>::Da as DaSpec>::BlockHeader,
        _validity_condition: &<<S as Spec>::Da as DaSpec>::ValidityCondition,
        config: &Self::Config,
        state: &mut impl GenesisState<Self::Spec>,
    ) -> Result<(), ModuleError> {
        self.admin.set(&config.sequencer_authority, state).unwrap();

        let now = SystemTime::now();
        let unix_timestamp: u64 = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        self.unix_timestamp.set(&unix_timestamp, state).unwrap();
        self.slot.set(&0, state).unwrap();

        Ok(())
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        state: &mut impl TxState<Self::Spec>,
    ) -> Result<(), ModuleError> {
        let call_result = match msg {
            CallMessage::UpdateSlot {} => self.update_slot(context, state),
            CallMessage::UpdateTimestamp {} => self.update_timestamp(context, state),
        };

        Ok(call_result?)
    }
}
