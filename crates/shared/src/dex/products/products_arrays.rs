use serde;
use std::ops::{Deref, DerefMut};

use crate::dex::{Product, MAX_PRODUCTS};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
#[repr(transparent)]
pub struct ProductsArray {
    // #[cfg_attr(feature = "native", serde(with = "serde_arrays"))]
    // pub array: [Product; MAX_PRODUCTS],
    pub array: Vec<Product>,
}

impl Default for ProductsArray {
    fn default() -> Self {
        ProductsArray { array: vec![] }
    }
}

impl Deref for ProductsArray {
    type Target = Vec<Product>;

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl DerefMut for ProductsArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}