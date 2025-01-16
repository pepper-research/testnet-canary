// //! The covariance metadata is the main tool which provides key datapoints used to construct the covariance matrix.
// //! For example, the covariance metadata provides a standard deviations array which is used in the computation of the covariance matrix.
// 
// use std::mem::size_of;
// 
// use spicenet_shared::fast_int::FastInt;
// 
// use crate::{error::RiskError, utils::RiskStateTag};
// use crate::temp::products::ProductId;
// 
// pub const MAX_OUTRIGHTS: usize = 128;
// 
// pub struct CovarianceMetadata {
//     /// State verifier for variance cache.
//     /// [`RiskStateTag::VarianceCache]
//     pub state_identifer: RiskStateTag,
// 
//     /// The last time that the variance cache was updated
//     pub update_offset: u64,
// 
//     /// The number of active products(not settled/expired) products
//     pub num_active_products: usize,
// 
//     /// Array of product keys of the type [`ProductId`]
//     pub product_keys: [ProductId; MAX_OUTRIGHTS],
// 
//     /// Array of standard deviations of the products, of the type [`FastInt`]
//     pub standard_deviations: [FastInt; MAX_OUTRIGHTS],
// }
// 
// impl CovarianceMetadata {
//     pub const LEN: usize = size_of::<CovarianceMetadata>();
// 
//     pub fn get_product_index(&self, product_key: &ProductId) -> Result<usize, RiskError> {
//         for index in 0..self.num_active_products {
//             if self.product_keys[index] == *product_key {
//                 return Ok(index);
//             }
//         }
// 
//         Err(RiskError::MissingCovarianceEntry.into())
//     }
// 
//     pub fn get_std(&self, product_key: &ProductId) -> Result<FastInt, RiskError> {
//         let index = self.get_product_index(product_key)?;
// 
//         Ok(self.standard_deviations[index])
//     }
// }
