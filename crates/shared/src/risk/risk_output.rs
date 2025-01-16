use borsh::{BorshDeserialize, BorshSerialize};
// use aaob::Side;
// use borsh::{BorshDeserialize, BorshSerialize};
use sov_modules_api::Spec;

use crate::{Fractional, Side, ZERO_FRAC};

use crate::health_status::HealthStatus;
use crate::risk::constants::MAX_OUTRIGHTS;
use crate::risk::constants::MAX_TRADER_POSITIONS;

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(BorshDeserialize, BorshSerialize, Eq)]
pub struct RiskEngineOutput {
    pub health_output: HealthOutput,
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(BorshDeserialize, BorshSerialize, Eq)]
pub enum HealthOutput {
    Healthy {
        health_status: HealthTracker,
    },
    Liquidatable {
        liquidation_status: LiquidationStatus,
    },
}

#[derive(Clone, PartialEq, Debug, BorshDeserialize, BorshSerialize, Eq)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
pub struct LiquidationStatus {
    pub health_status: HealthStatus,
    pub action_result: ActionStatus,
    pub total_social_loss: Fractional,
    pub liquidation_price: Fractional,
    // pub social_losses: [SocialLossInfo<S>; MAX_TRADER_POSITIONS],
    pub social_losses: Vec<SocialLossInfo>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(BorshDeserialize, BorshSerialize, Eq)]
pub struct HealthTracker {
    pub health_status: HealthStatus,

    pub action_status: ActionStatus,
}

#[derive(Copy, Clone, PartialEq, Debug, BorshDeserialize, BorshSerialize, Eq)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
pub enum ActionStatus {
    Approved,
    NotApproved,
}

#[derive(Copy, Clone, PartialEq, Debug, BorshDeserialize, BorshSerialize, Eq)]
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
pub struct SocialLossInfo {
    pub product_idx: usize,
    pub amount: Fractional,
}

impl Default for SocialLossInfo {
    fn default() -> Self {
        Self {
            product_idx: 0,
            amount: ZERO_FRAC,
        }
    }
}

impl SocialLossInfo {
    pub fn is_social_loss(&self) -> bool {
        self.product_idx < MAX_OUTRIGHTS && self.amount != ZERO_FRAC // amount can be negative.
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct RiskInfo {
    pub op_type: RiskEngineOpCodes,
    pub num_orders: u8,
    pub orders: [OrderRiskInfo; 12], // max 12 open orders
}

impl Default for RiskInfo {
    fn default() -> Self {
        Self {
            num_orders: 0,
            orders: [OrderRiskInfo::default(); 12],
            op_type: RiskEngineOpCodes::CheckHealth,
        }
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Copy)]
pub struct OrderRiskInfo {
    pub side: Side,
    pub order_price: Fractional,
    pub is_combo: bool,
    pub idx: usize,
}

impl Default for OrderRiskInfo {
    fn default() -> Self {
        Self {
            side: Side::Bid,
            order_price: ZERO_FRAC,
            is_combo: false,
            idx: 0,
        }
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
#[borsh(use_discriminant = true)]
pub enum RiskEngineOpCodes {
    NewOrder = 1,
    CancelOrder = 2,
    CheckHealth = 3,
    PositionTransfer = 4,
    ConsumeEvents = 5, // deprecated opcode
    CheckWithdrawHealth = 6,
    LockCollateral = 7,
    SignPT = 8, // signs a print trade quote.
}
