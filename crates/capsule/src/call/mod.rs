use crate::WalletType;
use anyhow::{bail, Result};
use sov_modules_api::{Address, Context, EventEmitter, Spec, TxState};

use crate::state::wallet::{Scope, ScopeVec};
use crate::{
    event::Event,
    state::wallet::{Wallet, WalletState},
    Capsule,
};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(bound = "Address<S>: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[cfg_attr(
    feature = "arbitrary",
    derive(arbitrary::Arbitrary, proptest_derive::Arbitrary)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, Eq, PartialEq, Clone)]
pub enum CallMessage<S: Spec> {
    CreateWallet {
        wallet_type: WalletType,
        signature: Vec<u8>,
        nonce: u64,
    },
    AddAdminWallet {
        address: Address<S>,
        approving_wallet: Wallet<S>,
        admin_wallet: WalletType,
        signature: Vec<u8>,
        nonce: u64,
    },
    AddEphemeralWallet {
        address: Address<S>,
        approving_wallet: Wallet<S>,
        ephemeral_wallet: WalletType,
        scopes: ScopeVec,
        expiration_timestamp: u64,
        signature: Vec<u8>,
        nonce: u64,
    },
    AddRecoveryWallet {
        address: Address<S>,
        approving_wallet: Wallet<S>,
        recovery_wallet: WalletType,
        signature: Vec<u8>,
        nonce: u64,
    },
    RevokeWallet {
        address: Address<S>,
        approving_wallet: Wallet<S>,
        wallet_type: WalletType,
        signature: Vec<u8>,
        nonce: u64,
    },
}

impl<S: Spec> Capsule<S> {
    pub fn create_wallet(
        &self,
        wallet_type: WalletType,
        signature: Vec<u8>,
        nonce: u64,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        self.throw_duplicate_wallet_error(&wallet_type, state)?;

        let (wallet_state, wallet) =
            WalletState::create_wallet(wallet_type, signature.as_ref(), nonce)?;

        self.smart_wallets
            .set(&wallet_state.address, &wallet_state, state)?;
        self.wallets.set(&wallet.wallet_type, &wallet, state)?;

        self.emit_event(
            state,
            Event::WalletCreated {
                master_wallet: wallet,
                address: wallet_state.address,
            },
        );

        Ok(())
    }

    pub fn add_admin_wallet(
        &self,
        address: Address<S>,
        approving_wallet: Wallet<S>,
        admin_wallet: WalletType,
        signature: Vec<u8>,
        nonce: u64,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        self.throw_duplicate_wallet_error(&admin_wallet, state)?;

        match self.smart_wallets.get(&address, state)? {
            Some(mut existing_wallet) => {
                let wallet = existing_wallet.add_admin_wallet(
                    &address,
                    &approving_wallet,
                    admin_wallet,
                    signature.as_ref(),
                    nonce,
                )?;
                self.smart_wallets.set(&address, &existing_wallet, state)?;
                self.wallets.set(&wallet.wallet_type, &wallet, state)?;

                self.emit_event(state, Event::WalletAdded { wallet, address });
            }
            None => bail!("Smart wallet not found"),
        }

        Ok(())
    }

    pub fn add_ephemeral_wallet(
        &self,
        address: Address<S>,
        approving_wallet: Wallet<S>,
        ephemeral_wallet: WalletType,
        scopes: ScopeVec,
        expiration_timestamp: u64,
        signature: Vec<u8>,
        nonce: u64,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        self.throw_duplicate_wallet_error(&ephemeral_wallet, state)?;

        match self.smart_wallets.get(&address, state)? {
            Some(mut existing_wallet) => {
                let wallet = existing_wallet.add_ephemeral_wallet(
                    &address,
                    &approving_wallet,
                    ephemeral_wallet,
                    scopes,
                    expiration_timestamp,
                    signature.as_ref(),
                    nonce,
                )?;
                self.smart_wallets.set(&address, &existing_wallet, state)?;
                self.wallets.set(&wallet.wallet_type, &wallet, state)?;

                self.emit_event(state, Event::WalletAdded { wallet, address });
            }
            None => bail!("Smart wallet not found"),
        }

        Ok(())
    }

    pub fn add_recovery_wallet(
        &self,
        address: Address<S>,
        approving_wallet: Wallet<S>,
        recovery_wallet: WalletType,
        signature: Vec<u8>,
        nonce: u64,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        self.throw_duplicate_wallet_error(&recovery_wallet, state)?;

        match self.smart_wallets.get(&address, state)? {
            Some(mut existing_wallet) => {
                let wallet = existing_wallet.add_recovery_wallet(
                    &address,
                    &approving_wallet,
                    recovery_wallet,
                    signature.as_ref(),
                    nonce,
                )?;
                self.smart_wallets.set(&address, &existing_wallet, state)?;
                self.wallets.set(&wallet.wallet_type, &wallet, state)?;

                self.emit_event(state, Event::WalletAdded { wallet, address });
            }
            None => bail!("Smart wallet not found"),
        }

        Ok(())
    }

    pub fn revoke_wallet(
        &self,
        address: Address<S>,
        approving_wallet: Wallet<S>,
        wallet_type: WalletType,
        signature: Vec<u8>,
        nonce: u64,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        match self.smart_wallets.get(&address, state)? {
            Some(mut existing_wallet) => {
                let message = WalletState::revoke_wallet_message(&address, &wallet_type, nonce)?;
                if !approving_wallet.verify_signature(signature.as_ref(), &message)? {
                    bail!("Invalid signature");
                }

                match self.wallets.get(&wallet_type, state)? {
                    Some(mut wallet) => {
                        wallet.revoked = true;
                        self.wallets.set(&wallet_type, &wallet, state)?;

                        self.emit_event(state, Event::WalletRevoked { wallet, address });
                    }
                    None => bail!("Wallet not found"),
                }
            }
            None => bail!("Smart wallet not found"),
        }

        Ok(())
    }
}
