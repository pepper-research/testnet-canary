use sov_modules_api::Spec;

use spicenet_shared::Fractional;

use crate::state::dex::TwoIterators;
use crate::state::{ComboProduct, DexError, OutrightProduct, ProductId, ProductStatus};
use spicenet_aaob::OrderbookId;
// use crate::state::IsInitialized;
use crate::state::dex::ProductTrait;
use crate::state::dex::mpg::MarketProductGroup;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
#[repr(C, u64)]
pub enum Product<S: Spec> {
    Outright { outright_product: OutrightProduct<S> },
    Combo { combo_product: ComboProduct<S> },
}

impl<S: Spec> Product<S> {
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

    pub fn try_to_combo(&self) -> std::result::Result<&ComboProduct<S>, DexError> {
        match self {
            Product::Outright {
                outright_product: _,
            } => Err(DexError::ProductNotCombo.into()),
            Product::Combo { combo_product: c } => Ok(c),
        }
    }

    pub fn try_to_outright(&self) -> std::result::Result<&OutrightProduct<S>, DexError> {
        match self {
            Product::Outright {
                outright_product: o,
            } => Ok(o),
            Product::Combo { combo_product: _ } => Err(DexError::ProductNotOutright.into()),
        }
    }

    pub fn try_to_combo_mut(&mut self) -> std::result::Result<&mut ComboProduct<S>, DexError> {
        match self {
            Product::Outright {
                outright_product: _,
            } => Err(DexError::ProductNotCombo.into()),
            Product::Combo { combo_product: c } => Ok(c),
        }
    }

    pub fn try_to_outright_mut(&mut self) -> std::result::Result<&mut OutrightProduct<S>, DexError> {
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
    pub fn get_product_status(&self, mpg_min: &MarketProductGroup<S>) -> ProductStatus {
        match self {
            Product::Outright { outright_product } => *outright_product.product_status,
            Product::Combo { combo_product } => combo_product.get_combo_status(mpg_min),
        }
    }
}

impl<S: Spec> ProductTrait for &Product<S> {
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
            Product::Outright { outright_product } => false,
            Product::Combo { combo_product } => true,
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
