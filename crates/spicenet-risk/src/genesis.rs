use serde::{Deserialize, Serialize};
use sov_modules_api::{Error, GenesisState, Spec};

use anyhow::Result;

use super::RiskModule;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct RiskModuleConfig {}

impl<S: Spec> RiskModule<S> {
    pub(crate) fn init_module(
        &self,
        _config: &<Self as sov_modules_api::Module>::Config,
        _state: &mut impl GenesisState<S>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
