// //! The risk profile provides an overview of a portfolio's risk levels with the help of various datapoints
// 
// use crate::{
//     health_status::HealthStatus,
//     // utils::babylonian_sqrt,
//     // variance_cache::VarianceCache,
//     variance_cache::MAX_TRADER_POSITIONS,
// };
// use spicenet_shared::fast_int::{FastInt, ZERO_FAST_INT};
// 
// /// The minimum threshold required to place open orders
// pub const ORDER_PLACEMENT_SDS: FastInt = FastInt {
//     value: 3_000_000_i128,
// };
// 
// /// The minimum threshold to be safe from liquidation
// pub const LIQUIDATION_SDS: FastInt = FastInt {
//     value: 1_500_000_i128,
// };
// 
// /// The liquidation price is set to LIQUIDATION_PRICE_PROPORTION * portfolio_value
// pub const LIQUIDATION_PRICE_PROPORTION: FastInt = FastInt {
//     value: 333_333_i128,
// };
// 
// #[derive(Debug)]
// pub struct RiskProfile {
//     /// The total amount of cash available in the account
//     pub net_cash: FastInt,
// 
//     /// The current PnL of the account
//     pub pnl: FastInt,
// 
//     /// Current market value of the trader's active positions
//     pub position_value: FastInt,
// 
//     /// Combined portfolio value, encompassing active position value + current cash value
//     pub portfolio_value: FastInt,
// 
//     /// One-day expected change(in STD) of portfolio value
//     pub portfolio_std_dev: FastInt,
// 
//     /// One-day expected change(in STD) if the largest open order on bid or offer for each instrument is filled
//     pub portfolio_open_order_std_div: FastInt,
// 
//     /// Absolute portfolio value calculated by using absolute values of price*position, indexed by product_index
//     pub abs_position_value: [FastInt; MAX_TRADER_POSITIONS],
// 
//     /// Sum of the above array
//     pub total_abs_position_value: FastInt,
// 
//     /// Amount of deposited collateral
//     pub deposited_collateral: FastInt,
// }
// 
// impl RiskProfile {
//     /// The threshold, which when broken, can cause the account to be deemed liquidatable
//     pub fn get_liquidation_threshold(&self) -> FastInt {
//         self.portfolio_std_dev * LIQUIDATION_SDS
//     }
// 
//     /// Calculates liquidation threshold as a % of portfolio value. Higher the ratio, higher the risk.
//     pub fn get_risk_ratio(&self) -> FastInt {
//         self.get_liquidation_threshold() / self.portfolio_value
//     }
// 
//     /// The threshold until which open orders can be allowed
//     pub fn get_order_placement_threshold(&self) -> FastInt {
//         self.portfolio_open_order_std_div / ORDER_PLACEMENT_SDS
//     }
// 
//     pub fn get_health_status(&self) -> HealthStatus {
//         let liquidation_value = self.get_liquidation_threshold();
//         let health_value = self.get_order_placement_threshold();
// 
//         if self.portfolio_value < liquidation_value {
//             HealthStatus::Liquidatable
//         } else if self.portfolio_value < health_value {
//             HealthStatus::Unhealthy
//         } else {
//             HealthStatus::Healthy
//         }
//     }
// }
// 
// // TODO (@karthik-pepperdex): implement From<(&VarianceCache, &TRG)> for RiskProfile to construct RiskProfile from VarianceCache and TRG
