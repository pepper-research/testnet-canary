use sokoban::NodeAllocatorMap;
use spicenet_shared::Side;

use crate::{MarketId, Slab, StateType};

pub type OrderbookId = MarketId;
pub type OrderId = u128; // TODO: cross check

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct OrderbookState {
    pub market_id: MarketId,
    pub bids: Slab,
    pub asks: Slab,
}

impl OrderbookState {
    pub fn find_bbo(&self, side: Side) -> Option<u32> {
        match side {
            Side::Bid => self.bids.find_max(),
            Side::Ask => self.asks.find_min(),
        }
    }
    pub fn get_spread(&self) -> (Option<u64>, Option<u64>) {
        let best_bid_price = self.bids.find_max().map(|x| {
            self.bids
                .tree
                .get(&self.bids.get_node(x).unwrap().key)
                .unwrap()
                .price
        });
        let best_ask_price = self.asks.find_max().map(|x| {
            self.asks
                .tree
                .get(&self.asks.get_node(x).unwrap().key)
                .unwrap()
                .price
        });
        (best_bid_price, best_ask_price)
    }

    pub fn get_tree(&mut self, side: Side) -> &mut Slab {
        // TODO: (anishde12020) remove lifetimes
        match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        }
    }
    pub fn commit_to_header(self, market_address: MarketId) {
        self.bids
            .write_header(None, market_address, StateType::Bids);
        self.asks
            .write_header(None, market_address, StateType::Asks);
    }

    pub fn is_empty(&self) -> bool {
        self.asks.root().is_none() && self.bids.root().is_none()
    }
}
