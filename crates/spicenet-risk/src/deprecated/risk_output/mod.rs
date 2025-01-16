// use crate::covariance_metadata::MAX_OUTRIGHTS;
// use crate::health_status::HealthStatus;
// use crate::variance_cache::MAX_TRADER_POSITIONS;
// // use aaob::Side;
// // use borsh::{BorshDeserialize, BorshSerialize};
// use serde::{Deserialize, Serialize};
// // use spicenet_shared::fractional::{Fractional, ZERO_FRAC};
// use borsh::{BorshDeserialize, BorshSerialize};
// use spicenet_shared::{Fractional, ZERO_FRAC, Side};
//
// pub struct RiskEngineOutput {
//     pub health_output: HealthOutput,
// }
//
// #[derive(
//     Copy, Clone, PartialEq, Debug, Serialize, Deserialize,
// )]
// pub enum HealthOutput {
//     Healthy {
//         health_status: HealthTracker,
//     },
//     Liquidatable {
//         liquidation_status: LiquidationStatus,
//     },
// }
//
// #[derive(
//     Copy, BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug, Serialize, Deserialize,
// )]
// pub struct LiquidationStatus {
//     pub health_status: HealthStatus,
//     pub action_result: ActionStatus,
//     pub total_social_loss: Fractional,
//     pub liquidation_price: Fractional,
//     pub social_losses: [SocialLossInfo; MAX_TRADER_POSITIONS],
// }
//
// #[derive(
//     Copy, BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug, Serialize, Deserialize,
// )]
// pub struct HealthTracker {
//     pub health_status: HealthStatus,
//
//     pub action_status: ActionStatus,
// }
//
// #[derive(BorshDeserialize, BorshSerialize, Clone, Copy, PartialEq, Debug)]
// pub enum ActionStatus {
//     Approved,
//     NotApproved,
// }
//
// #[derive(
//     Copy, BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug, Serialize, Deserialize,
// )]
// pub struct SocialLossInfo {
//     pub product_idx: usize,
//     pub amount: Fractional,
// }
//
// impl Default for SocialLossInfo {
//     fn default() -> Self {
//         Self {
//             product_idx: 0,
//             amount: ZERO_FRAC,
//         }
//     }
// }
//
// impl SocialLossInfo {
//     pub fn is_social_loss(&self) -> bool {
//         self.product_idx < MAX_OUTRIGHTS && self.amount != ZERO_FRAC // amount can be negative.
//     }
// }
//
// #[derive(
//     Copy, BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug, Serialize, Deserialize,
// )]
// pub struct RiskInfo {
//     pub op_type: RiskEngineOpCodes,
//     pub num_orders: u8,
//     pub orders: [OrderRiskInfo; 12], // max 12 open orders
// }
//
// impl Default for RiskInfo {
//     fn default() -> Self {
//         Self {
//             num_orders: 0,
//             orders: [Default::default(); 12],
//             op_type: RiskEngineOpCodes::CheckHealth,
//         }
//     }
// }
//
// #[derive(
//     Copy, BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug, Serialize, Deserialize,
// )]
// pub struct OrderRiskInfo {
//     pub side: Side,
//     pub order_price: Fractional,
//     pub is_combo: bool,
//     pub idx: usize,
// }
//
// impl Default for OrderRiskInfo {
//     fn default() -> Self {
//         Self {
//             side: Side::Bid,
//             order_price: ZERO_FRAC,
//             is_combo: false,
//             idx: 0,
//         }
//     }
// }
//
// #[derive(
//     Copy, BorshDeserialize, BorshSerialize, Clone, PartialEq, Debug, Serialize, Deserialize,
// )]
// pub enum RiskEngineOpCodes {
//     NewOrder = 1,
//     CancelOrder = 2,
//     CheckHealth = 3,
//     PositionTransfer = 4,
//     ConsumeEvents = 5, // deprecated opcode
//     CheckWithdrawHealth = 6,
//     LockCollateral = 7,
//     SignPT = 8, // signs a print trade quote.
// }
