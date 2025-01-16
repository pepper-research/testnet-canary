//! The demo-rollup supports `EVM` and `sov-module` authenticators.
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sov_modules_api::capabilities::{
    AuthenticationError, AuthenticationOutput, AuthorizationData, FatalError,
    UnregisteredAuthenticationError,
};
use sov_modules_api::runtime::capabilities::TransactionAuthenticator;
use sov_modules_api::transaction::TransactionWithoutCall;
use sov_modules_api::{DispatchCall, ProvableStateReader, RawTx, Spec};
use sov_state::User;

use crate::chain_hash::CHAIN_HASH;
use crate::runtime::{Runtime, RuntimeCall};

impl<S: Spec> TransactionAuthenticator<S> for Runtime<S> {
    type Decodable = <Self as DispatchCall>::Decodable;

    type AuthorizationData = AuthorizationData<S>;

    type Input = Auth;

    type Signature = Auth<TransactionWithoutCall<S>>;

    fn parse_input(
        &self,
        tx: &Self::Input,
    ) -> Result<(Self::Decodable, Self::Signature), FatalError> {
        match tx {
            Auth::Mod(raw_tx) => {
                let (call, tx) = sov_modules_api::capabilities::parse_input::<_, Self>(raw_tx)?;
                Ok((call, Auth::Mod(tx)))
            }
        }
    }

    fn authenticate<Accessor: ProvableStateReader<User, Spec = S>>(
        &self,
        input: &Self::Input,
        pre_exec_ws: &mut Accessor,
    ) -> Result<
        AuthenticationOutput<S, Self::Decodable, Self::AuthorizationData>,
        AuthenticationError,
    > {
        match input {
            Auth::Mod(tx) => sov_modules_api::capabilities::authenticate::<_, S, Self>(
                tx,
                &CHAIN_HASH,
                pre_exec_ws,
            ),
        }
    }

    fn authenticate_unregistered<Accessor: ProvableStateReader<User, Spec = S>>(
        &self,
        input: &Self::Input,
        pre_exec_ws: &mut Accessor,
    ) -> Result<
        AuthenticationOutput<S, Self::Decodable, Self::AuthorizationData>,
        UnregisteredAuthenticationError,
    > {
        let contents = match input {
            Auth::Mod(tx) => tx,
        };

        let (tx_and_raw_hash, auth_data, runtime_call) =
            sov_modules_api::capabilities::authenticate::<_, S, Runtime<S>>(
                contents,
                &CHAIN_HASH,
                pre_exec_ws,
            )
            .map_err(|e| match e {
                AuthenticationError::FatalError(err, hash) => {
                    UnregisteredAuthenticationError::FatalError(err, hash)
                }
                AuthenticationError::OutOfGas(err) => {
                    UnregisteredAuthenticationError::OutOfGas(err)
                }
            })?;

        match &runtime_call {
            RuntimeCall::SequencerRegistry(sov_sequencer_registry::CallMessage::Register {
                ..
            }) => Ok((tx_and_raw_hash, auth_data, runtime_call)),
            _ => Err(UnregisteredAuthenticationError::FatalError(
                FatalError::Other(
                    "The runtime call included in the transaction was invalid.".to_string(),
                ),
                tx_and_raw_hash.raw_tx_hash,
            ))?,
        }
    }

    fn add_standard_auth(tx: RawTx) -> Self::Input {
        Auth::Mod(tx.data)
    }
}

#[derive(Debug, PartialEq, Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub enum Auth<Mod = Vec<u8>> {
    /// Authenticate using the `EVM` authenticator, which expects a standard EVM transaction
    /// (i.e. an rlp-encoded payload signed using secp256k1 and hashed using keccak256).
    // Evm(Evm),
    /// Authenticate using the standard `sov-module` authenticator, which uses the default
    /// signature scheme and hashing algorithm defined in the rollup's [`Spec`].
    Mod(Mod),
}
