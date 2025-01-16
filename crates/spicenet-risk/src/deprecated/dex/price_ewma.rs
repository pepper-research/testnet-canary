use sov_modules_api::Spec;

use spicenet_shared::Fractional;

pub const NO_BID: Fractional = Fractional {
    m: i64::MIN,
    exp: 0,
};

pub const NO_ASK: Fractional = Fractional {
    m: i64::MAX,
    exp: 0,
};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct PriceEwma<S: Spec> {
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

impl<S: Spec> PriceEwma<S> {
    pub fn initialize(&mut self, slot: u64) {
        self.slot = *slot;
        for ewma in self.ewma_bid.iter_mut() {
            *ewma = NO_BID;
        }

        for ewma in self.ewma_ask.iter_mut() {
            *ewma = NO_ASK;
        }

        self.bid = NO_BID;
        self.ask = NO_ASK;

        self.prev_bid = NO_BID;
        self.prev_ask = NO_ASK;
    }

    pub fn get_bbo(&self) -> Option<Fractional> {
        match (self.bid == NO_BID, self.ask == NO_ASK) {
            (true, true) => None,
            (true, false) => Some(self.ask),
            (false, true) => Some(self.bid),
            (false, false) => Some((self.ask + self.bid) / Fractional::TWO_FRAC),
        }
    }
}
