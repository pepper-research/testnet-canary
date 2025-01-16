use serde::{Deserialize, Serialize};
use sov_bank::Bank;
use sov_modules_api::{
    Address, CallResponse, Context, Error, GenesisState, Module, ModuleId, ModuleInfo, Spec,
    StateMap, StateValue, TxState,
};
use spicenet_time::TimeModule;

use crate::call::CallMessage;
use crate::event::Event;
use crate::state::{OracleNode, WhitelistedUser};

pub mod call;
pub mod event;
#[cfg(feature = "native")]
pub mod rpc;
pub mod state;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "Address<S>: Serialize + serde::de::DeserializeOwned")]
pub struct OracleRegistryConfig<S: Spec> {
    pub registry_authority: Address<S>,
    pub minimum_bond_amt: u64,
}

#[derive(Clone, ModuleInfo, sov_modules_api::macros::ModuleRestApi)]
pub struct OracleRegistry<S: Spec> {
    #[id]
    pub(crate) id: ModuleId,

    #[module]
    pub(crate) bank: Bank<S>,
    #[module]
    pub(crate) time: TimeModule<S>,

    #[state]
    pub(crate) oracle_nodes: StateMap<Address<S>, OracleNode<S>>,

    #[state]
    pub(crate) whitelisted_users: StateMap<Address<S>, WhitelistedUser<S>>,

    #[state]
    pub(crate) registry_authority: StateValue<Address<S>>,

    #[state]
    pub(crate) minimum_bond_amt: StateValue<u64>,
}

impl<S: Spec> Module for OracleRegistry<S> {
    type Spec = S;

    type Config = OracleRegistryConfig<S>;

    type CallMessage = CallMessage<S>;

    type Event = Event<S>;

    fn genesis(
        &self,
        config: &Self::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<(), Error> {
        self.registry_authority
            .set(&config.registry_authority, state)
            .unwrap();
        self.minimum_bond_amt
            .set(&config.minimum_bond_amt, state)
            .unwrap();

        Ok(())
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse, Error> {
        let call_result = match msg {
            CallMessage::Register {
                node_address,
                user_address,
                amount,
            } => self.register(node_address, user_address, amount, context, state),
            CallMessage::Deposit {
                node_address,
                user_address,
                amount,
            } => self.deposit(node_address, user_address, amount, context, state),
            CallMessage::Exit {
                node_address,
                user_address,
            } => self.exit(node_address, user_address, context, state),
            CallMessage::Whitelist { user_address } => self.whitelist(user_address, context, state),
        };

        Ok(call_result?)
    }
}
