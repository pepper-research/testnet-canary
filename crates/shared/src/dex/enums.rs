use crate::dex::ProductStatus;
use bytemuck::{Pod, Zeroable};
use sov_modules_api::Spec;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
#[repr(u64)]
pub enum AccountTag {
    Uninitialized,
    MarketProductGroup,
    TraderRiskGroup,
    TraderPosition,
    MarketProductGroupWithCombos,
    ComboGroup,
    Combo,
    RiskProfile,
    LockedCollateral,
}

impl Default for AccountTag {
    fn default() -> Self {
        AccountTag::Uninitialized
    }
}

unsafe impl Zeroable for AccountTag {}

impl Copy for AccountTag {}

unsafe impl Pod for AccountTag {}

impl AccountTag {
    pub fn to_bytes(&self) -> [u8; 8] {
        match self {
            AccountTag::Uninitialized => 0_u64.to_le_bytes(),
            AccountTag::MarketProductGroup => 1_u64.to_le_bytes(),
            AccountTag::TraderRiskGroup => 2_u64.to_le_bytes(),
            AccountTag::TraderPosition => 3_u64.to_le_bytes(),
            AccountTag::MarketProductGroupWithCombos => 4_u64.to_le_bytes(),
            AccountTag::ComboGroup => 5_u64.to_le_bytes(),
            AccountTag::Combo => 6_u64.to_le_bytes(),
            AccountTag::RiskProfile => 7_u64.to_le_bytes(),
            AccountTag::LockedCollateral => 8_u64.to_le_bytes(),
        }
    }
}

// #[cfg_attr(
//     feature = "native",
//     derive(serde::Serialize),
//     derive(serde::Deserialize)
// )]
// #[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq, Eq, Hash)]
// #[repr(u64)]
// pub enum ProductStatus {
//     Uninitialized,
//     Initialized,
//     Expired,
//     Expiring,
// }

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
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OrderType {
    Limit,
    ImmediateOrCancel,
    FillOrKill,
    PostOnly,
}
