// //! The variance cache is the main gadget which determines the overall portfolio risk and allows careful regulation of portfolio risk.
// //! It uses various datapoints which give an idea of portfolio risk which can be converted into dynamic maintenance margin requirements which are reflective
// //! of portfolio risk. I.e more the variance, more the maintenance margin requirements, hence responsibly regulating risk.
// 
// use std::mem::size_of;
// 
// use spicenet_shared::fast_int::FastInt;
// 
// use crate::utils::RiskStateTag;
// 
// pub const MAX_TRADER_POSITIONS: usize = 16;
// 
// // #[zero_copy]
// pub struct VarianceCache {
//     /// State verifier for variance cache.
//     /// [`RiskStateTag::VarianceCache]
//     pub state_verifier: RiskStateTag,
// 
//     /// The last time that the variance cache was updated
//     /// This is used to determine if the VC needs to be re-built in case the covariance matrix changes.
//     pub update_offset: u64,
// 
//     /// The market value of a trader's position as of the last risk update.
//     /// As the name suggests, this only includes derivatives and not cash positions.
//     pub derivative_position_value: FastInt,
// 
//     /// Total variance of the traded position
//     pub total_variance_traded: FastInt,
// 
//     /// Additional variance induced by (any) open orders.
//     pub open_order_variance: FastInt,
// 
//     /// Product indexes as of the last variance cache calculation
//     pub product_indexes: [usize; 2 * MAX_TRADER_POSITIONS],
// 
//     /// Positions as of the last variance cache calculation
//     pub positions: [FastInt; 2 * MAX_TRADER_POSITIONS],
// 
//     /// Equates to covariance multiplied by positions as of the last variance cache calculation
//     pub sigma_position: [FastInt; 2 * MAX_TRADER_POSITIONS],
// 
//     /// TODO: add documentation
//     pub total_liquidity_buffer: FastInt,
// }
// 
// impl VarianceCache {
//     pub const LEN: usize = size_of::<VarianceCache>();
// }
