use std::marker::PhantomData;

use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sov_modules_api::GenesisState;
use sov_modules_api::Spec;

use super::{get_market_id, Market, AAOB};

/// Initial configuration for AAOB module.

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    serde(bound = "Market<S>: Serialize + serde::de::DeserializeOwned")
)]
#[derive(Debug, Clone, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub struct AAOBConfig<S: Spec> {
    initial_markets: Vec<Market<S>>,
    #[cfg_attr(
        feature = "native",
        serde(skip_serializing, default)
    )]
    _phantom: PhantomData<S>,
}

impl<S: Spec> AAOB<S> {
    /// Initializes module, doing nothing special right now
    pub(crate) fn init_module(
        &self,
        config: &<Self as sov_modules_api::Module>::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<()> {
        // Initialize markets
        for market in config.initial_markets.iter() {
            let address = get_market_id::<S>(&market.name);
            self.markets.set(&address, market, state)?;
        }

        Ok(())
    }
}

// Dummy test for now
#[cfg(test)]
mod tests {
    #[test]
    fn test_config_serialization() {
        assert_eq!(true, true);
    }
}
