use sov_modules_api::Spec;
use spicenet_aaob::OrderbookId;
use spicenet_shared::{Fractional, NAME_LEN};

// use crate::state::dex::PriceEwma;
use crate::state::ProductId;
use crate::state::price_ewma::PriceEwma;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct ProductMetadata<S: Spec> {
    /// Product ID
    pub product_id: ProductId,

    /// Name of the product represented where each character is represented by a `u8` type with `NAME_LEN` number of characters per product.
    pub name: [u8; NAME_LEN],

    /// Orderbook id represented as a u64
    pub orderbook_id: OrderbookId,

    /// The tick size of a product defined at initialization.
    pub tick_size: Fractional,

    /// Base decimals of a product defined at initialization.
    pub base_decimals: u64,

    /// TODO
    pub price_offset: Fractional,

    /// Total volume traded in notional terms
    pub notional_traded_volume: Fractional,

    /// Set of important prices of the product, such as the EWMA bid, EWMA ask and so on.
    pub prices: PriceEwma<S>,
}
