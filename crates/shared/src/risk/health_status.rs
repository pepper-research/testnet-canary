use borsh::{BorshDeserialize, BorshSerialize};
// TODO: merge into risk engine metadata file(WIP)
use serde::{Deserialize, Serialize};

#[derive(
    Copy, Serialize, Deserialize, Clone, PartialEq, Debug, BorshDeserialize, BorshSerialize, Eq,
)]
#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
pub enum HealthStatus {
    Healthy,
    /// If true, allows for all open orders to be cancelled BUT doesn't allow for positions to be transferred(liquidated)
    Unhealthy,
    /// If true, allows for all open orders to be cancelled AND allows for position to be transferred AND any new post only orders are blocked.
    Liquidatable,
    NotLiquidatable,
}
