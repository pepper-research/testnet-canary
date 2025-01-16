use std::fmt::Display;

// use crate::state::pubkey::Pubkey;
use anyhow::{anyhow, bail, Result};
use borsh::BorshDeserialize;
use sov_modules_api::{Address, Spec, StateAccessor, StateMap};
use spicenet_shared::addresses::TrgId;
use std::fmt::Formatter;

use spicenet_shared::crypto::{
    ed25519::verify_signature as verify_ed25519_signature,
    ethereum::verify_signature as verify_ethereum_signature,
};

use rand::rngs::OsRng;
use rand::RngCore;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(sov_modules_api::macros::UniversalWallet),
    serde(bound = "Address<S>: serde::Serialize + serde::de::DeserializeOwned")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct WalletState<S: Spec> {
    pub address: Address<S>,
    pub wallets: Vec<WalletType>,
    pub trgs: Vec<TrgId<S>>,
}

impl<S: Spec> WalletState<S> {
    pub fn wallets(
        &self,
        wallets: &StateMap<WalletType, Wallet<S>>,
        state: &mut impl StateAccessor,
    ) -> Vec<Wallet<S>> {
        self.wallets
            .iter()
            .filter_map(|w| wallets.get(w, state).ok().flatten())
            .collect()
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(bound = "S::Address: ::schemars::JsonSchema", rename = "Wallet"),
    serde(bound = "Address<S>: serde::Serialize + serde::de::DeserializeOwned")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, PartialEq, Eq, Clone, Debug)]
pub struct Wallet<S: Spec> {
    pub wallet_type: WalletType,
    pub revoked: bool,
    pub role: Role,
    pub smart_wallet: Address<S>,
}

// #[cfg(feature = "native")]
// impl<S: Spec> JsonSchema for Pubkey {
//     fn schema_name() -> String {
//         "Pubkey".to_string()
//     }

//     fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
//         gen.subschema_for::<[u8; 32]>()
//     }
// }

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Eq, Clone)]
pub enum WalletType {
    Solana { address: [u8; 32] },
    Aptos { address: [u8; 32] },
    Ethereum { address: [u8; 20] }, // without 0x prefix
    Sui { address: [u8; 32] },      // without 0x prefix, only ed25119 for now
}
// TODO: add other wallet types

impl Display for WalletType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletType::Solana { address } => {
                write!(f, "solana:{}", bs58::encode(address).into_string())
            }
            WalletType::Aptos { address } => write!(f, "aptos:0x{}", hex::encode(address)),
            WalletType::Ethereum { address } => write!(f, "ethereum:0x{}", hex::encode(address)),
            WalletType::Sui { address } => write!(f, "sui:0x{}", hex::encode(address)),
        }
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub enum Role {
    Admin,
    Ephemeral {
        expiration_timestamp: u64,
        scopes: ScopeVec,
    },
    Recovery,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Eq, Clone)]
pub enum Scope {
    Trading,
    Funds,
}
// TODO: add more scopes

impl Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::Trading => write!(f, "trading"),
            Scope::Funds => write!(f, "funds"),
        }
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct ScopeVec(Vec<Scope>);

impl ScopeVec {
    pub fn contains(&self, scope: &Scope) -> bool {
        self.0.contains(scope)
    }
}

impl Display for ScopeVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl<S: Spec> WalletState<S> {
    pub fn create_wallet(
        wallet_type: WalletType,
        signature: &[u8],
        nonce: u64,
    ) -> Result<(WalletState<S>, Wallet<S>)> {
        let mut rand_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut rand_bytes);
        let address = Address::try_from_slice(&rand_bytes)?; // TODO: maybe use a better logic for this

        let wallet = Wallet {
            wallet_type,
            revoked: false,
            role: Role::Admin,
            smart_wallet: address.clone(),
        };

        let message = Self::create_wallet_message(&wallet.wallet_type, nonce)?;

        if !wallet.verify_signature(signature, &message)? {
            bail!("Invalid signature");
        }

        let wallet_state = WalletState {
            address,
            wallets: vec![wallet.clone().wallet_type],
            trgs: vec![],
        };

        Ok((wallet_state, wallet))
    }

    pub fn add_admin_wallet(
        &mut self,
        address: &Address<S>,
        approving_wallet: &Wallet<S>,
        admin_wallet: WalletType,
        signature: &[u8],
        nonce: u64,
    ) -> Result<Wallet<S>> {
        let message = Self::add_admin_wallet_message(address, &admin_wallet, nonce)?;
        if !approving_wallet.verify_signature(signature, &message)? {
            bail!("Invalid signature");
        }

        let wallet = Wallet {
            wallet_type: admin_wallet,
            revoked: false,
            role: Role::Admin,
            smart_wallet: address.clone(),
        };

        self.wallets.push(wallet.clone().wallet_type);

        Ok(wallet)
    }

    pub fn add_ephemeral_wallet(
        &mut self,
        address: &Address<S>,
        approving_wallet: &Wallet<S>,
        ephemeral_wallet: WalletType,
        scopes: ScopeVec,
        expiration_timestamp: u64,
        signature: &[u8],
        nonce: u64,
    ) -> Result<Wallet<S>> {
        let message = Self::add_ephemeral_wallet_message(
            address,
            &ephemeral_wallet,
            &scopes,
            expiration_timestamp,
            nonce,
        )?;
        if !approving_wallet.verify_signature(signature, &message)? {
            bail!("Invalid signature");
        }

        let wallet = Wallet {
            wallet_type: ephemeral_wallet,
            revoked: false,
            role: Role::Ephemeral {
                scopes,
                expiration_timestamp,
            },
            smart_wallet: address.clone(),
        };

        self.wallets.push(wallet.clone().wallet_type);

        Ok(wallet)
    }

    pub fn add_recovery_wallet(
        &mut self,
        address: &Address<S>,
        approving_wallet: &Wallet<S>,
        recovery_wallet: WalletType,
        signature: &[u8],
        nonce: u64,
    ) -> Result<Wallet<S>> {
        let message = Self::add_recovery_wallet_message(address, &recovery_wallet, nonce)?;
        if !approving_wallet.verify_signature(signature, &message)? {
            bail!("Invalid signature");
        }

        let wallet = Wallet {
            wallet_type: recovery_wallet,
            revoked: false,
            role: Role::Recovery,
            smart_wallet: address.clone(),
        };

        self.wallets.push(wallet.clone().wallet_type);

        Ok(wallet)
    }

    // pub fn revoke_wallet(
    //     &mut self,
    //     address: &Address<S>,
    //     wallet_type: WalletType,
    //     signature: &[u8],
    //     nonce: u64,
    // ) -> Result<()> {
    //     let message = Self::revoke_wallet_message(address, wallet_type, nonce);
    //     if !self.verify_signature(signature, message)? {
    //         bail!("Invalid signature");
    //     }

    //     Ok(())
    // }
}

impl<S: Spec> Wallet<S> {
    pub fn is_allowed(&self, scope: Scope) -> bool {
        match &self.role {
            Role::Ephemeral { scopes, .. } => scopes.contains(&scope),
            _ => true,
        }
    }

    pub fn is_expired(&self, timestamp: u64) -> bool {
        match self.role {
            Role::Ephemeral {
                expiration_timestamp,
                ..
            } => expiration_timestamp < timestamp,
            _ => false,
        }
    }

    pub fn is_active(&self, timestamp: u64) -> bool {
        !self.revoked && !self.is_expired(timestamp)
    }

    pub fn verify_signature(&self, signature: &[u8], message: &[u8]) -> Result<bool> {
        match self.wallet_type {
            WalletType::Solana { address } => {
                Ok(verify_ed25519_signature(signature, message, &address).is_ok())
            }
            WalletType::Aptos { address } => {
                Ok(verify_ed25519_signature(signature, message, &address).is_ok())
            }
            WalletType::Ethereum { address } => {
                Ok(verify_ethereum_signature(signature, message, &address).is_ok())
            }
            WalletType::Sui { address } => {
                Ok(verify_ed25519_signature(signature, message, &address).is_ok())
            }
        }
    }
}

/// message spec
/// Create wallet - "I am creating a new smart wallet and adding an admin wallet {type:address}. Nonce: {nonce}"
/// Add admin wallet - "I am adding an admin wallet {type:address} to {address}"
/// Add ephemeral wallet - "I am adding an ephemeral wallet {type:address} to {address} with scopes {scopes} and expiration timestamp {expiration_timestamp}. Nonce: {nonce}"
/// Add recovery wallet - "I am adding a recovery wallet {type:address} to {address}. Nonce: {nonce}"
/// Revoke wallet - "I am revoking the wallet {type:address} of {address}. Nonce: {nonce}"

impl<S: Spec> WalletState<S> {
    pub fn create_wallet_message(master_wallet: &WalletType, nonce: u64) -> Result<[u8; 1024]> {
        format!(
            "I am creating a new smart wallet and adding an admin wallet {master_wallet}. Nonce: {nonce}"
        )
            .as_bytes()
            .try_into()
            .map_err(|err| anyhow!("Failed to convert message to bytes, {:?}", err))
    }

    pub fn add_admin_wallet_message(
        address: &Address<S>,
        admin_wallet: &WalletType,
        nonce: u64,
    ) -> Result<[u8; 1024]> {
        format!("I am adding an admin wallet {admin_wallet} to {address}. Nonce: {nonce}")
            .as_bytes()
            .try_into()
            .map_err(|_| anyhow!("Failed to convert message to bytes"))
    }

    pub fn add_ephemeral_wallet_message(
        address: &Address<S>,
        ephemeral_wallet: &WalletType,
        scopes: &ScopeVec,
        expiration_timestamp: u64,
        nonce: u64,
    ) -> Result<[u8; 1024]> {
        format!(
            "I am adding an ephemeral wallet {ephemeral_wallet} to {address} with scopes {scopes} and expiration timestamp {expiration_timestamp}. Nonce: {nonce}"
        )
            .as_bytes()
            .try_into()
            .map_err(|_| anyhow!("Failed to convert message to bytes"))
    }

    pub fn add_recovery_wallet_message(
        address: &Address<S>,
        recovery_wallet: &WalletType,
        nonce: u64,
    ) -> Result<[u8; 1024]> {
        format!("I am adding a recovery wallet {recovery_wallet} to {address}. Nonce: {nonce}")
            .as_bytes()
            .try_into()
            .map_err(|_| anyhow!("Failed to convert message to bytes"))
    }

    pub fn revoke_wallet_message(
        address: &Address<S>,
        wallet_type: &WalletType,
        nonce: u64,
    ) -> Result<[u8; 1024]> {
        format!("I am revoking the wallet {wallet_type} of {address}. Nonce: {nonce}")
            .as_bytes()
            .try_into()
            .map_err(|_| anyhow!("Failed to convert message to bytes"))
    }
}
