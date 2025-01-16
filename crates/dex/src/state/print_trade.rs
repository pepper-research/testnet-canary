use borsh::{BorshDeserialize, BorshSerialize};
use sov_modules_api::{Address, Spec};

use spicenet_shared::{Fractional, MPGId, ProductId, Side};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct PrintTradeProduct<S: Spec> {
    pub product_key: ProductId, // verify that the product at the given index is this one
    pub size: Fractional,       // quantity of base (e.g. BTCUSD contract)
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct PrintTradeProductIndex {
    pub product_index: usize,
    pub size: Fractional, // quantity of base (e.g. BTCUSD contract)
}

// #[derive(AnchorSerialize, AnchorDeserialize, Default, Debug)] not allowed on arrays
pub type PrintTradeProducts<S: Spec> = [PrintTradeProduct<S>; PrintTrade::MAX_PRODUCTS_PER_TRADE];
// #[derive(AnchorSerialize, AnchorDeserialize, Default, Debug)] not allowed on arrays
pub type PrintTradeProductIndexes = [PrintTradeProductIndex; PrintTrade::MAX_PRODUCTS_PER_TRADE];

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct PrintTrade<S: Spec> {
    pub is_initialized: bool,
    pub creator: Address<S>,
    pub counterparty: Address<S>, // TODO: Unsure what it should be for now, dummy
    pub seed: Address<S>,         // TODO: Unsure what it should be for now, dummy
    pub market_product_group: MPGId, // technically might not need to store this
    pub num_products: usize,
    pub products: PrintTradeProducts<S>,
    pub price: Fractional, // quantity of quote (USD) per base
    pub side: Side,
    pub operator: Address<S>, // TODO: Unsure what it should be for now, dummy
    pub operator_creator_fee_proportion: Fractional,
    pub operator_counterparty_fee_proportion: Fractional,
    pub is_signed: bool,
    pub is_cancelled: CancelStatus,
    pub bump: u8,
}

impl<S: Spec> PrintTrade<S> {
    pub const MAX_PRODUCTS_PER_TRADE: usize = 6;
    pub const SIZE: usize = std::mem::size_of::<PrintTrade<S>>();
}

#[derive(BorshSerialize, BorshDeserialize, Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum CancelStatus {
    Active,
    CreatorCancelled,
    CounterpartyCancelled,
}
