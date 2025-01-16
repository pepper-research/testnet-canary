use sov_modules_api::{
    CallResponse, Context, Error, GenesisState, Module, ModuleId, ModuleInfo, Spec, StateMap,
    StateValue, StateVec, TxState,
};

use spicenet_shared::dex::{MarketProductGroup, TraderRiskGroup};

pub use call::*;
pub use event::*;
pub use genesis::*;
pub use rpc::*;
use spicenet_risk::RiskModule;
use spicenet_shared::addresses::TrgId;
use spicenet_shared::MPGId;
pub use state::*;
pub use utils::*;

pub mod call;
pub mod event;
pub mod genesis;
mod rpc;
pub mod state;
pub mod utils;

#[derive(Clone, ModuleInfo, sov_modules_api::macros::ModuleRestApi)]
pub struct Dex<S: Spec> {
    #[id]
    id: ModuleId,

    #[module]
    pub(crate) risk_engine: RiskModule<S>,

    #[state]
    pub trader_risk_groups: StateMap<TrgId<S>, TraderRiskGroup<S>>,

    #[state]
    pub market_product_groups: StateMap<MPGId, MarketProductGroup<S>>,
}

impl<S: Spec> Module for Dex<S> {
    type Spec = S;
    type Config = DexConfig<S>;
    type CallMessage = CallMessage<S>;
    type Event = Event;

    fn genesis(
        &self,
        config: &Self::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<(), Error> {
        Ok(self.init_module(config, state)?)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse, Error> {
        let call_result = match msg {
            CallMessage::InitTrg {} => self.initialize_trg(),
            CallMessage::UpdateProductFunding {
                amount,
                new_product_status,
            } => self.update_product_funding(amount, new_product_status, context, state),
        };

        Ok(call_result?)
    }
}
