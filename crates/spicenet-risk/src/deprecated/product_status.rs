// product status
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
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

unsafe impl Pod for ProductStatus {}
unsafe impl Zeroable for ProductStatus {}
