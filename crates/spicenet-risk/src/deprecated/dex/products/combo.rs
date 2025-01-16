use std::ops::{Deref, DerefMut};

use sov_modules_api::Spec;

use crate::state::{MAX_LEGS, ProductMetadata};

use super::ProductStatus;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct ComboProduct<S: Spec> {
    pub metadata: ProductMetadata<S>,
    pub num_legs: usize,
    pub legs_array: [ComboLeg<S>; MAX_LEGS],
}
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct ComboLeg<S: Spec> {
    pub product_index: usize,
    pub product_key: ProductId,
    pub ratio: i64,
}

impl<S: Spec> ComboProduct<S> {
    pub fn legs(&self) -> &[ComboLeg<S>] {
        &self.legs_array[..self.num_legs]
    }

    pub fn has_leg(&self, product_key: ProductId) -> bool {
        self.legs()
            .iter()
            .take(self.num_legs)
            .any(|l| l.product_key == product_key)
    }

    // TODO: add MPG
    // TODO: pub fn is_expired()

    /// [`get_combo_status`] returns the status of a combo product listed on an MPG.
    pub fn get_combo_status(&self, mpg_min: &MarketProductGroupMin) -> ProductStatus {
        // We first obtain the result of calling the `find_product_index_among_all` function, which returns
        // the product id and the Product itself.
        let result = mpg_min.find_product_index_among_all(&self.metadata.product_id);

        // If the result returns an `Ok()`, we return a block, otherwise we return
        // [`ProductStatus::Uninitialized`]
        match result {
            Ok(_) => {}
            Err(_) => return ProductStatus::Uninitialized, // if err, product doesn't exist
        }

        // Then, we proceed to check if the legs of the combo product is initialized, or not.
        // For that, we first construct the outright product from the [`ComboLeg`] struct, using the `find_outright_among_all` fn.
        // Then, we return the product status. However, while constructing the outright, we do not find any outright,
        // we return Uninitialized.
        for leg in self.legs() {
            if let Ok((_, outright)) = mpg_min.find_outright_among_all(&leg.product_key) {
                if outright.product_status != ProductStatus::Initialized {
                    return outright.product_status;
                }
            } else {
                return ProductStatus::Uninitialized;
            }
        }
        ProductStatus::Initialized
    }
}

impl<S: Spec> Deref for ComboProduct<S> {
    type Target = ProductMetadata<S>;

    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl<S: Spec> DerefMut for ComboProduct<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.metadata
    }
}

impl<S: Spec> ProductTrait for &ComboProduct<S> {
    #[inline]
    fn get_product_key(&self) -> &ProductId {
        &self.metadata.product_id
    }

    #[inline]
    fn is_combo(&self) -> bool {
        true
    }

    #[inline]
    fn get_name(&self) -> &[u8; 16] {
        &self.metadata.name
    }

    #[inline]
    fn get_orderbook_id(&self) -> &OrderbookId {
        &self.metadata.orderbook_id
    }
}
