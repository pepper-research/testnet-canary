use crate::{MarketId, OrderId, OrderbookId};

use spicenet_shared::utils::Side;

/// Dummy event
#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone,
)]
pub enum Event {
    OrderCreated {
        order_id: OrderId,
        market_id: MarketId,
        side: Side,
        total_base_qty: u64,
        total_quote_qty: u64,
        total_base_qty_posted: u64,
    },
    OrderCancelled {
        order_id: OrderId,
        market_id: MarketId,
        side: Side,
    },
    MarketCreated {
        market_id: MarketId,
        market_name: String,
    },
    MarketClosed {
        market_id: MarketId,
        market_name: String,
    },
}
