use sov_modules_api::{
    CallResponse, Context, Error, GenesisState, Module, ModuleId, ModuleInfo, Spec, StateMap,
    TxState,
};

pub use call::*;
pub use error::*;
pub use event::*;
pub use genesis::*;
use lut::LookupTable;
use spicenet_dex::Dex;
use spicenet_shared::derivative::DerivativeID;
use spicenet_time::TimeModule;
pub use state::*;
pub use utils::*;

use crate::derivative_metadata::DerivativeMetadata;

pub mod call;
pub mod error;
pub mod event;
pub mod genesis;
pub mod state;
pub mod utils;

#[derive(Clone, ModuleInfo, sov_modules_api::macros::ModuleRestApi)]
pub struct Instruments<S: Spec> {
    #[id]
    id: ModuleId,

    #[state]
    pub derivative_metadata: StateMap<DerivativeID, DerivativeMetadata<S>>,

    #[module]
    pub(crate) dex: Dex<S>,

    #[module]
    pub(crate) oracle: LookupTable<S>, // I think u should be using registry not lookup table,pls verify

    #[module]
    pub(crate) time: TimeModule<S>,
}

impl<S: Spec> Module for Instruments<S> {
    type Spec = S;
    type Config = InstrumentsConfig;
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
    ) -> Result<CallResponse, Error, DexError> {
        let call_result = match msg {
            CallMessage::InitDerivative { params } => {
                self.initialize_derivative(params, context, state)
            }
            CallMessage::CloseDerivative { params } => {
                self.close_derivative(params, context, state)
            }
            CallMessage::SettleDerivative { params } => {
                self.settle_derivative(params, context, state)
            }
        };

        Ok(call_result?)
    }
}
