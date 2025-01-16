use bytemuck::{Pod, Zeroable};
use crate::ProductStatus;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum ProductStatus {
    Uninitialized,
    Initialized,
    Expired,
    Expiring,
}

// impl Default for ProductStatus {
//     fn default() -> Self {
//         ProductStatus::Uninitialized
//     }
// }

unsafe impl Zeroable for ProductStatus {}

impl Copy for ProductStatus {}

unsafe impl Pod for ProductStatus {}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OrderType {
    Limit,
    ImmediateOrCancel,
    FillOrKill,
    PostOnly,
}