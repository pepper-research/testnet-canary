use std::ops::{Deref, DerefMut};
// use crate::{}
use crate::state::products::Product;
use spicenet_shared::MAX_PRODUCTS;
use sov_modules_api::Spec;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
#[repr(transparent)]
pub struct ProductsArray<S: Spec> {
    pub array: [Product<S>; MAX_PRODUCTS],
}

impl<S: Spec> Deref for ProductsArray<S> {
    type Target = [Product<S>; MAX_PRODUCTS];

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<S: Spec> DerefMut for ProductsArray<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}
