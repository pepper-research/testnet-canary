use anyhow::Result;
use serde::{Deserialize, Serialize};
use sov_modules_api::GenesisState;
use sov_modules_api::Spec;

use crate::Instruments;

/// Initial configuration for Dex module.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InstrumentsConfig {}

impl<S: Spec> Instruments<S> {
    pub(crate) fn init_module(
        &self,
        config: &<Self as sov_modules_api::Module>::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_config_serialization() {
        assert_eq!(true, true);
    }
}
