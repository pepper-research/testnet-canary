use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// [`OracleType`] enum returns the currently used oracle service by the exchange.
/// To preserve unformity, we would want to use only one oracle service across the exchange's service at any point in time.
/// Magnus - the primary oracle implementation which is designed towards feature-completion.
/// Orpheus - the backup oracle implementation which is designed towards serving raw oracle updates at speed.
#[derive(
    BorshSerialize, BorshDeserialize, Serialize, Deserialize, Copy, Debug, Clone, PartialEq,
)]
#[repr(u64)]
pub enum OracleType {
    /// Hot oracle storage.
    /// Primary oracle that is emptied, and updated every rollup slot.
    /// Works on a set of keeper nodes that generate a price for every supported feed once every rollup slot.
    /// Assuming N set of keeper nodes, and X feeds, we have N*X prices every rollup slot. Meaning that, for every feed, we have N different prices.
    /// Nodes send their prices to an aggregator service which is responsible for aggregating the N prices for every X[i] into one price, thereby reducing the number of prices to X prices, i.e one price for every feed.
    /// Then, the aggregator posts these prices into the oracle module, which is structured as an LUT allowing for fast writes and cheap lookups.
    Magnus,
    /// Cold oracle storage.
    /// We would want to maintain only one price for every feed, i.e for X number of feeds, we would want X number of prices stored in the LUT only, thereby not degrading the performance of the array structure.
    /// Meaning that, before writing new price data to the LUT, we first need to make sure it is empty. Every rollup slot, we first empty the LUT and send the previous slot's prices to a WAL and insert new price data into the LUT.
    /// The WAL then writes these historical prices into a raw K-V optimized for fast reads.
    Orpheus,
}

impl Default for OracleType {
    fn default() -> Self {
        OracleType::Magnus
    }
}

unsafe impl Pod for OracleType {}
unsafe impl Zeroable for OracleType {}
