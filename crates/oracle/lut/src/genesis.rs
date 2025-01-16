use crate::{fixed_ring_buffer::FixedRingBuffer, LookupTable};
use serde::{Deserialize, Serialize};
use sov_modules_api::{Address, Genesis, GenesisState, ModuleError, Spec};
use spicenet_shared::{oracle::NUMBER_OF_MARKETS, ZERO_FRAC};

/// Config for the LookupTable module. (used for genesis purpose)
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    serde(bound = "Address<S>: Serialize + serde::de::DeserializeOwned")
)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LookupTableConfig<S: Spec> {
    /// Existing prices.
    pub prices: [u64; NUMBER_OF_MARKETS],
    /// Existing aggregate confidence intervals.
    pub aggregate_conf_intervals: [u32; NUMBER_OF_MARKETS],
    pub update_authority: Address<S>,
}

impl<S: Spec> LookupTable<S> {
    pub(crate) fn init_module(
        &self,
        config: &<LookupTable<S> as Genesis>::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<(), ModuleError> {
        let prices = [ZERO_FRAC; NUMBER_OF_MARKETS];
        let aggregate_conf_intervals = [0u32; NUMBER_OF_MARKETS];
        self.prices.set(&prices, state);
        self.aggregate_conf_intervals
            .set(&aggregate_conf_intervals, state);

        let price_ticks = FixedRingBuffer::default();
        let aggregate_conf_interval_ticks = FixedRingBuffer::default();

        self.price_ticks.set(&price_ticks, state);
        self.aggregate_conf_interval_ticks
            .set(&aggregate_conf_interval_ticks, state);

        self.update_authority.set(&config.update_authority, state);

        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn test_config_serialization() {
//         assert_eq!(true, true);
//     }
// }
