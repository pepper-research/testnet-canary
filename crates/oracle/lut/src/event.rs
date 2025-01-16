use spicenet_shared::{oracle::NUMBER_OF_MARKETS, Fractional};

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
    // Event emitted when a mutation takes place for prices
    //
    // Fields:
    // - 'prices' - Array of prices
    // - 'aggregate_conf_intervals' - Array of aggregate confidence intervals in same order of corresponding prices
    MutateAll {
        prices: [Fractional; NUMBER_OF_MARKETS],
        aggregate_conf_intervals: [u32; NUMBER_OF_MARKETS],
    },
}
