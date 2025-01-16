use crate::aaob::OrderbookId;
use crate::Fractional;
use crate::ProductId;
use sov_modules_api::Spec;

use crate::dex::{PriceEwma, NAME_LEN};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct ProductMetadata {
    /// Product ID
    pub product_id: ProductId,

    /// Price index
    pub price_index: usize,

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
    pub prices: PriceEwma,
}
