// #![allow(dead_code)]
// //! Implementation of a covariance matrix used to calculate the variance of a cross-margined portfolio.
// //! Variance is an key enabler of SPANDEX as it allows for determining the maintenance margin of a portfolio.
// //! I.e, higher the variance, higher the overall maintenance margin required to maintain the portfolio.
// 
// use spicenet_shared::fast_int::FastInt;
// 
// use crate::{
//     // address::ProductAddress,
//     correlation_matrix::CorrelationMatrix,
//     covariance_metadata::{CovarianceMetadata, MAX_OUTRIGHTS},
//     error::RiskError,
//     temp::products::ProductId,
// };
// 
// pub const MAX_PRODUCTS: usize = 256;
// 
// pub struct CovarianceMatrix<'a> {
//     pub covariance_metadata: &'a CovarianceMetadata,
// 
//     pub correlations: &'a CorrelationMatrix,
// 
//     mappings: [u16; MAX_PRODUCTS],
// }
// 
// impl<'cm> CovarianceMatrix<'cm> {
//     pub fn new(
//         metadata: &'cm CovarianceMetadata,
//         correlations: &'cm CorrelationMatrix,
//         mappings: [u16; MAX_PRODUCTS],
//     ) -> Self {
//         Self {
//             covariance_metadata: metadata,
//             correlations,
//             mappings,
//         }
//     }
// 
//     pub fn get_covariance(
//         &self,
//         pubkey_1: &ProductId,
//         pubkey_2: &ProductId,
//     ) -> Result<FastInt, RiskError> {
//         if *pubkey_1 == *pubkey_2 {
//             let product_index = self.covariance_metadata.get_product_index(pubkey_1)?;
//             return self.get_covariance_from_product_indexes(product_index, product_index);
//         }
// 
//         self.get_covariance_from_product_indexes(
//             self.covariance_metadata.get_product_index(pubkey_1)?,
//             self.covariance_metadata.get_product_index(pubkey_2)?,
//         )
//     }
// 
//     pub fn get_covariance_from_product_indexes(
//         &self,
//         product_index_1: usize,
//         product_index_2: usize,
//     ) -> Result<FastInt, RiskError> {
//         if product_index_1 == product_index_2 {
//             let std = self.covariance_metadata.standard_deviations
//                 [self.mappings[product_index_1] as usize];
//             Ok(std * std)
//         } else {
//             let mapped_index_1 = self.mappings[product_index_1] as usize;
//             let mapped_index_2 = self.mappings[product_index_2] as usize;
// 
//             let (lesser, greater) = match mapped_index_1 < mapped_index_2 {
//                 true => (mapped_index_1, mapped_index_2),
//                 false => (mapped_index_2, mapped_index_1),
//             };
// 
//             let correlation = self.correlations.get_corr_unchecked(lesser, greater);
//             Ok(self.covariance_metadata.standard_deviations[mapped_index_1]
//                 * self.covariance_metadata.standard_deviations[mapped_index_2]
//                 * correlation)
//         }
//     }
// }
// 
// pub struct MutableCovarianceMatrix<'a> {
//     pub covariance_metadata: &'a mut CovarianceMetadata,
//     pub correlations: &'a mut CorrelationMatrix,
// }
// 
// impl<'cm> MutableCovarianceMatrix<'cm> {
//     pub fn to_covariance_matrix(&self, mappings: [u16; MAX_PRODUCTS]) -> CovarianceMatrix {
//         CovarianceMatrix::new(self.covariance_metadata, self.correlations, mappings)
//     }
// 
//     pub fn set_covariance(
//         &mut self,
//         product_keys: &[ProductId],
//         std: &Vec<f32>,
//         correlations: &Vec<Vec<f32>>,
//     ) -> Result<(), RiskError> {
//         let len = product_keys.len();
// 
//         if len > MAX_OUTRIGHTS {
//             return Err(RiskError::InvalidCovarianceInput.into());
//         }
// 
//         if len != std.len() {
//             return Err(RiskError::InvalidCovarianceInput.into());
//         }
// 
//         if len != correlations.len() {
//             return Err(RiskError::InvalidCovarianceInput.into());
//         }
// 
//         for correlation in correlations.iter() {
//             if len != correlation.len() {
//                 return Err(RiskError::InvalidCovarianceInput.into());
//             }
//         }
// 
//         self.covariance_metadata.num_active_products = len;
// 
//         for (idx, product_key) in product_keys.iter().enumerate() {
//             self.covariance_metadata.product_keys[idx] = **product_key.clone()
//         }
// 
//         for (idx, std_dev) in std.iter().enumerate() {
//             self.covariance_metadata.standard_deviations[idx] = (*std_dev).into()
//         }
// 
//         self.correlations.num_active_products = len;
//         for (index_i, correlation_row) in correlations.iter().enumerate() {
//             for (index_j, correlation) in correlation_row.iter().enumerate().skip(index_i) {
//                 self.correlations.set_corr(index_i, index_j, *correlation)?;
//             }
//         }
// 
//         Ok(())
//     }
// }
