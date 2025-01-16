use sov_modules_api::{Address, Spec};

#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone,
)]
#[serde(bound = "Address<S>: serde::Serialize + serde::de::DeserializeOwned")]
pub enum Event<S: Spec> {
    NodeRegistered {
        node_address: Address<S>,
        amount: u64,
    },
    NodeDeposited {
        node_address: Address<S>,
        amount: u64,
    },
    NodeExited {
        node_address: Address<S>,
    },
    UserWhitelisted {
        user_address: Address<S>,
    },
}
