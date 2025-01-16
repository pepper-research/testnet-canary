use sov_modules_api::Spec;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
#[repr(u64)]
pub enum ProductStatus {
    Uninitialized,
    Initialized,
    Expired,
    Expiring,
}

impl Default for ProductStatus {
    fn default() -> Self {
        ProductStatus::Uninitialized
    }
}
