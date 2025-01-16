use sov_modules_api::{Address, Spec};

use crate::state::wallet::Wallet;

#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone
)]
#[serde(
    bound = "Address<S>: serde::Serialize + serde::de::DeserializeOwned, Wallet<S>: serde::Serialize + serde::de::DeserializeOwned"
)]
pub enum Event<S: Spec> {
    WalletCreated {
        master_wallet: Wallet<S>,
        address: Address<S>,
    },
    WalletAdded {
        wallet: Wallet<S>,
        address: Address<S>,
    },
    WalletRevoked {
        wallet: Wallet<S>,
        address: Address<S>,
    },
}
