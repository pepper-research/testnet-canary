use crate::WalletType;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorCode;
use sov_modules_api::macros::rpc_gen;
use sov_modules_api::prelude::UnwrapInfallible;
use sov_modules_api::{Address, ApiStateAccessor, Spec};

use crate::state::wallet::WalletState;
use crate::{state::wallet::Wallet, Capsule};

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(
    bound = "WalletState<S>: serde::Serialize + serde::de::DeserializeOwned, Wallet<S>: serde::Serialize + serde::de::DeserializeOwned"
)]
pub struct WalletResponse<S: Spec> {
    pub smart_wallet: WalletState<S>,
    pub wallets: Vec<Wallet<S>>,
}

#[rpc_gen(client, server, namespace = "capsule")]
impl<S: Spec> Capsule<S> {
    #[rpc_method(name = "getWallet")]
    pub fn get_wallet(
        &self,
        address: Address<S>,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<WalletResponse<S>> {
        let smart_wallet = self
            .smart_wallets
            .get(&address, state)
            .unwrap_infallible()
            .ok_or(ErrorCode::InvalidParams)?;
        let wallets = smart_wallet.wallets(&self.wallets, state);

        Ok(WalletResponse {
            smart_wallet,
            wallets,
        })
    }

    #[rpc_method(name = "getCorrespondingSmartWallet")]
    pub fn get_corresponding_smart_wallet(
        &self,
        wallet_type: WalletType,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<WalletResponse<S>> {
        let wallet = self
            .wallets
            .get(&wallet_type, state)
            .unwrap_infallible()
            .ok_or(ErrorCode::InvalidParams)?;
        let smart_wallet = self
            .smart_wallets
            .get(&wallet.smart_wallet, state)
            .unwrap_infallible()
            .ok_or(ErrorCode::InvalidParams)?;
        let wallets = smart_wallet.wallets(&self.wallets, state);

        Ok(WalletResponse {
            smart_wallet,
            wallets,
        })
    }
}