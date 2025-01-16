use sov_modules_api::Spec;

use crate::{Fractional, TWO_FRAC};

use crate::{NO_ASK_PRICE, NO_BID_PRICE};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Eq)]
pub struct PriceEwma {
    /// Exponentially-Weighted Moving Average(EWMA) of the Bid
    pub ewma_bid: [Fractional; 4],
    /// Exponentially-Weighted Moving Average(EWMA) of the Ask
    pub ewma_ask: [Fractional; 4],

    /// The slot at which [`PriceEwma`] was last updated
    pub slot: u64,

    /// The current best bid
    pub bid: Fractional,
    /// The current best ask
    pub ask: Fractional,

    /// The best bid of the last slot
    pub prev_bid: Fractional,
    /// The best ask of the last slot
    pub prev_ask: Fractional,
}

impl PriceEwma {
    pub fn initialize(&mut self, slot: u64) {
        self.slot = slot;
        for ewma in self.ewma_bid.iter_mut() {
            *ewma = NO_BID_PRICE;
        }

        for ewma in self.ewma_ask.iter_mut() {
            *ewma = NO_ASK_PRICE;
        }

        self.bid = NO_BID_PRICE;
        self.ask = NO_ASK_PRICE;

        self.prev_bid = NO_BID_PRICE;
        self.prev_ask = NO_ASK_PRICE;
    }

    pub fn get_bbo(&self) -> Option<Fractional> {
        match (self.bid == NO_BID_PRICE, self.ask == NO_ASK_PRICE) {
            (true, true) => None,
            (true, false) => Some(self.ask),
            (false, true) => Some(self.bid),
            (false, false) => Some((self.ask + self.bid) / TWO_FRAC),
        }
    }
}
