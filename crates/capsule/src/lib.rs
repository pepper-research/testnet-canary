use anyhow::Result;
// use jsonrpsee::core::Serialize;
use serde::Deserialize;
use call::CallMessage;
use event::Event;
use serde::Serialize;
use sov_modules_api::{
    Address, Context, Error, GenesisState, Module, ModuleId, ModuleInfo, Spec,
    StateMap, StateValue, TxState,
};
use sov_rollup_interface::da::DaSpec;
use state::wallet::{Wallet, WalletState, WalletType};

pub mod call;
pub mod event;
#[cfg(feature = "native")]
pub mod rpc;
pub mod state;
pub mod utils;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleConfig {}

#[derive(Clone, ModuleInfo, sov_modules_api::ModuleRestApi)]
pub struct Capsule<S: Spec> {
    #[id]
    /// The ID of the capsule module.
    id: ModuleId,

    #[state]
    smart_wallets: StateMap<Address<S>, WalletState<S>>,

    #[state]
    wallets: StateMap<WalletType, Wallet<S>>,
}

impl<S: Spec> Module for Capsule<S> {
    type Spec = S;

    type Config = CapsuleConfig;

    type Event = Event<S>;

    type CallMessage = CallMessage<S>;

    fn genesis(
        &self,
        _genesis_rollup_header: &<<S as Spec>::Da as DaSpec>::BlockHeader,
        _validity_condition: &<<S as Spec>::Da as DaSpec>::ValidityCondition,
        _config: &Self::Config,
        _state: &mut impl GenesisState<S>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        state: &mut impl TxState<S>,
    ) -> Result<(), Error> {
        let call_result = match msg {
            CallMessage::CreateWallet {
                wallet_type,
                signature,
                nonce,
            } => self.create_wallet(wallet_type, signature, nonce, context, state),
            CallMessage::AddAdminWallet {
                address,
                approving_wallet,
                admin_wallet,
                signature,
                nonce,
            } => self.add_admin_wallet(
                address,
                approving_wallet,
                admin_wallet,
                signature,
                nonce,
                context,
                state,
            ),
            CallMessage::AddEphemeralWallet {
                address,
                approving_wallet,
                ephemeral_wallet,
                signature,
                scopes,
                expiration_timestamp,
                nonce,
            } => self.add_ephemeral_wallet(
                address,
                approving_wallet,
                ephemeral_wallet,
                scopes,
                expiration_timestamp,
                signature,
                nonce,
                context,
                state,
            ),
            CallMessage::AddRecoveryWallet {
                address,
                approving_wallet,
                recovery_wallet,
                signature,
                nonce,
            } => self.add_recovery_wallet(
                address,
                approving_wallet,
                recovery_wallet,
                signature,
                nonce,
                context,
                state,
            ),
            CallMessage::RevokeWallet {
                address,
                approving_wallet,
                wallet_type,
                signature,
                nonce,
            } => self.revoke_wallet(
                address,
                approving_wallet,
                wallet_type,
                signature,
                nonce,
                context,
                state,
            ),
        }?;

        Ok(call_result)
    }
}
