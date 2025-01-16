use sov_modules_api::{Address, Spec};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct OracleNode<S: Spec> {
    pub amount_staked: u64,
    pub accumulated_penalty: u64,
    pub address: Address<S>,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct WhitelistedUser<S: Spec> {
    pub address: Address<S>,
    pub whitelisted_ts: u64,

    /// Checks whether this whitelisted user is an oracle node yet or not.
    pub is_oracle_node: bool,
}
