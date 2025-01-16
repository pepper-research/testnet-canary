use crate::aaob::OrderbookId;
use crate::Fractional;
use crate::ProductId;
use sov_modules_api::Spec;

use std::ops::{Deref, DerefMut};

use crate::{
    ComboProduct, DexError, MarketProductGroup, OutrightProduct, ProductStatus, ProductTrait,
    TwoIterators,
};

use super::ProductMetadata;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
#[repr(C, u64)]
pub enum Product {
    Outright { outright_product: OutrightProduct },
    Combo { combo_product: ComboProduct },
}

impl Product {
    pub fn get_best_bid(&self) -> Fractional {
        match self {
            Product::Outright { outright_product } => outright_product.metadata.prices.bid,
            Product::Combo { combo_product } => combo_product.metadata.prices.bid,
        }
    }

    pub fn get_best_ask(&self) -> Fractional {
        match self {
            Product::Outright { outright_product } => outright_product.metadata.prices.ask,
            Product::Combo { combo_product } => combo_product.metadata.prices.ask,
        }
    }

    pub fn get_prev_best_bid(&self, slot: u64) -> Fractional {
        match self {
            Product::Outright { outright_product } => {
                if slot > outright_product.metadata.prices.slot {
                    outright_product.metadata.prices.bid
                } else {
                    outright_product.metadata.prices.prev_bid
                }
            }
            Product::Combo { combo_product } => {
                if slot > combo_product.metadata.prices.slot {
                    combo_product.metadata.prices.bid
                } else {
                    combo_product.metadata.prices.prev_bid
                }
            }
        }
    }

    pub fn get_prev_best_ask(&self, slot: u64) -> Fractional {
        match self {
            Product::Outright { outright_product } => {
                if slot > outright_product.metadata.prices.slot {
                    outright_product.metadata.prices.ask
                } else {
                    outright_product.metadata.prices.prev_ask
                }
            }
            Product::Combo { combo_product } => {
                if slot > combo_product.metadata.prices.slot {
                    combo_product.metadata.prices.ask
                } else {
                    combo_product.metadata.prices.prev_ask
                }
            }
        }
    }

    pub fn try_to_combo(&self) -> std::result::Result<&ComboProduct, DexError> {
        match self {
            Product::Outright {
                outright_product: _,
            } => Err(DexError::ProductNotCombo.into()),
            Product::Combo { combo_product: c } => Ok(c),
        }
    }

    pub fn try_to_outright(&self) -> std::result::Result<&OutrightProduct, DexError> {
        match self {
            Product::Outright {
                outright_product: o,
            } => Ok(o),
            Product::Combo { combo_product: _ } => Err(DexError::ProductNotOutright.into()),
        }
    }

    pub fn try_to_combo_mut(&mut self) -> std::result::Result<&mut ComboProduct, DexError> {
        match self {
            Product::Outright {
                outright_product: _,
            } => Err(DexError::ProductNotCombo.into()),
            Product::Combo { combo_product: c } => Ok(c),
        }
    }

    pub fn try_to_outright_mut(&mut self) -> std::result::Result<&mut OutrightProduct, DexError> {
        match self {
            Product::Outright {
                outright_product: o,
            } => Ok(o),
            Product::Combo { combo_product: _ } => Err(DexError::ProductNotOutright.into()),
        }
    }

    pub fn get_ratios_and_product_indexes(
        &self,
        idx: usize,
    ) -> impl Iterator<Item = (i64, usize)> + '_ {
        match self {
            Product::Outright {
                outright_product: _,
            } => TwoIterators::A(([(1, idx)]).into_iter()),
            Product::Combo { combo_product: c } => TwoIterators::B(
                c.legs()
                    .iter()
                    .take(c.num_legs)
                    .map(|leg| (leg.ratio, leg.product_index)),
            ),
        }
    }

    /// [`get_product_status`] returns the current product status listed on a Market Product Group.
    /// If the product is an outright, we directly return the `product_status` field of outrights.
    /// If the product is a combo product, we use the `get_combo_status` to obtain it's status.
    pub fn get_product_status<S: Spec>(&self, mpg_min: &MarketProductGroup<S>) -> ProductStatus {
        match self {
            Product::Outright { outright_product } => outright_product.product_status,
            Product::Combo { combo_product } => combo_product.get_combo_status(mpg_min),
        }
    }
}

impl ProductTrait for &Product {
    #[inline]
    fn get_product_key(&self) -> &ProductId {
        match self {
            Product::Outright { outright_product } => &outright_product.metadata.product_id,
            Product::Combo { combo_product } => &combo_product.metadata.product_id,
        }
    }

    #[inline]
    fn is_combo(&self) -> bool {
        match self {
            Product::Outright { .. } => false,
            Product::Combo { .. } => true,
        }
    }

    #[inline]
    fn get_name(&self) -> &[u8; 16] {
        match self {
            Product::Outright { outright_product } => &outright_product.metadata.name,
            Product::Combo { combo_product } => &combo_product.metadata.name,
        }
    }

    #[inline]
    fn get_orderbook_id(&self) -> &OrderbookId {
        match self {
            Product::Outright { outright_product } => &outright_product.metadata.orderbook_id,
            Product::Combo { combo_product } => &combo_product.metadata.orderbook_id,
        }
    }
}

impl DerefMut for Product {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Product::Outright {
                outright_product: x,
            } => &mut x.metadata,
            Product::Combo { combo_product: x } => &mut x.metadata,
        }
    }
}

impl Deref for Product {
    type Target = ProductMetadata;

    fn deref(&self) -> &Self::Target {
        match self {
            Product::Outright {
                outright_product: x,
            } => &x.metadata,
            Product::Combo { combo_product: x } => &x.metadata,
        }
    }
}

impl Default for Product {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}
