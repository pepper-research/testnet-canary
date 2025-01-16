use sov_modules_api::ModuleRestApi;
use sov_modules_api::{
    Address, Context, Error, Genesis, GenesisState, Module, ModuleId, ModuleInfo,
    Spec, StateValue, TxState, DaSpec
};
use spicenet_shared::{oracle::NUMBER_OF_MARKETS, Fractional};
use spicenet_time::TimeModule;
mod call;
mod event;
pub use crate::call::CallMessage;
mod rpc;
pub use rpc::*;
pub mod fixed_ring_buffer;
pub mod state;

mod genesis;
pub use genesis::*;

pub use crate::event::Event;
use crate::fixed_ring_buffer::FixedRingBuffer;

// #[cfg_attr(feature = "native")]
#[derive(Clone, ModuleInfo, ModuleRestApi)]
pub struct LookupTable<S: Spec> {
    #[id]
    id: ModuleId,

    #[state]
    prices: StateValue<[Fractional; NUMBER_OF_MARKETS]>,
    #[state]
    aggregate_conf_intervals: StateValue<[u32; NUMBER_OF_MARKETS]>,

    #[state]
    price_ticks: StateValue<FixedRingBuffer<[Fractional; NUMBER_OF_MARKETS], 3600>>,
    #[state]
    aggregate_conf_interval_ticks: StateValue<FixedRingBuffer<[u32; NUMBER_OF_MARKETS], 3600>>,

    #[state]
    price_emas: StateValue<[Fractional; NUMBER_OF_MARKETS]>,

    #[state]
    last_tick_timestamp: StateValue<u64>,

    #[state]
    update_authority: StateValue<Address<S>>,

    #[module]
    time_module: TimeModule<S>,
}

impl<S: Spec> Module for LookupTable<S> {
    type Spec = S;

    type Config = LookupTableConfig<S>;

    type CallMessage = CallMessage;

    type Event = Event;

    fn genesis(
        &self,
        _genesis_rollup_header: &<<S as Spec>::Da as DaSpec>::BlockHeader,
        _validity_condition: &<<S as Spec>::Da as DaSpec>::ValidityCondition,
        config: &Self::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<(), Error> {
        self.init_module(config, state)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error> {
        let call_result = match msg {
            CallMessage::MutateAll {
                prices,
                aggregate_conf_intervals,
            } => self.update_state(prices, aggregate_conf_intervals, context, state),
        };
        Ok(())
    }
}
