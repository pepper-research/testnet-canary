use sov_modules_api::Spec;

use crate::time::Slot;
use crate::FastInt;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct VarianceCache {
    // tag: RiskAccountTag, // since these are not Solana PDAs, we shouldn't need this
    pub update_offset: Slot,
    pub derivative_position_value: FastInt,
    pub total_variance_traded: FastInt,
    pub open_order_variance: FastInt,
    // product_indexes: [usize; 2 * MAX_TRADER_POSITIONS],
    // sigma_position: [FastInt; 2 * MAX_TRADER_POSITIONS],
    // positions: [FastInt; 2 * MAX_TRADER_POSITIONS],
    pub product_indexes: Vec<usize>,
    pub sigma_position: Vec<FastInt>,
    pub positions: Vec<FastInt>,
    pub total_liquidity_buffer: FastInt,
}
