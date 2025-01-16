#![allow(dead_code)]
//! Utility structs and enums

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    schemars(rename = "Side"),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Copy, Eq)]
pub enum Side {
    Bid,
    Ask,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }
}
