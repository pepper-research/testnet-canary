// use {
//     borsh::{BorshDeserialize, BorshSerialize},
//     bytemuck::{Pod, Zeroable},
//     crate::{
//         covariance_matrix::MAX_PRODUCTS,
//         error::DexError,
//         // fractional::{Fractional, ZERO_FRAC},
//         temp::{
//             mpg_minimal::MarketProductGroupMin, price_ewma::PriceEwma,
//             product_status::ProductStatus,
//         },
//         two_iterators::TwoIterators,
//     },
//     serde::{Deserialize, Serialize},
//     sov_modules_api::impl_hash32_type,
//     spicenet_shared::fractional::{Fractional, ZERO_FRAC},
//     std::ops::{Deref, DerefMut},
// };
// 
// // constants
// 
// pub const NAME_LEN: usize = 16;
// 
// pub const MAX_LEGS: usize = 4;
// 
// /// [`ProductMetadata`] defines crucial metadata for a product on the exchange.
// /// It can also be defined as shared detail between an outright product and a combo product.
// #[derive(
//     Debug, Eq, PartialEq, Pod, BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone, Copy,
// )]
// pub struct ProductMetadata {
//     /// Product ID
//     pub product_id: ProductId,
// 
//     /// Name of the product represented where each character is represented by a `u8` type with `NAME_LEN` number of characters per product.
//     pub name: [u8; NAME_LEN],
// 
//     /// Orderbook id represented as a u64
//     pub orderbook_id: OrderbookId,
// 
//     /// The tick size of a product defined at initialization.
//     pub tick_size: Fractional,
// 
//     /// Base decimals of a product defined at initialization.
//     pub base_decimals: u64,
// 
//     /// TODO
//     pub price_offset: Fractional,
// 
//     /// Total volume traded in notional terms
//     pub notional_traded_volume: Fractional,
// 
//     /// Set of important prices of the product, such as the EWMA bid, EWMA ask and so on.
//     pub prices: PriceEwma,
// }
// 
// unsafe impl Zeroable for ProductMetadata {}
// 
// #[derive(
//     Copy, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Eq, Debug,
//     , Default)]
// pub struct ProductId(u64);
// 
// impl PartialEq<ProductId> for ProductId {
//     fn eq(&self, other: &Self) -> bool {
//         self.0 == other.0
//     }
// }
// 
// #[derive(
//     Copy, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Eq, PartialEq, Debug,
// )]
// pub struct OrderbookId(u64);
// 
// impl OrderbookId {
//     pub fn new(&self) -> OrderbookId {
//         OrderbookId(0)
//     }
// 
//     // always use this
//     pub fn new_with_id(&self, id: u64) -> OrderbookId {
//         OrderbookId(id)
//     }
// }
// 
// impl ProductId {
//     pub fn new(&self) -> ProductId {
//         ProductId(0)
//     }
// 
//     // always use this
//     pub fn new_with_id(&self, id: u64) -> ProductId {
//         ProductId(id)
//     }
// }
// 
// /// [`OutrightProduct`] represents an outright instrument on the exchange.
// /// An outright can be defined as a product with a single leg.
// #[derive(
//     Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone, Copy,
// )]
// pub struct OutrightProduct {
//     /// The associated metadata of the outright product.
//     pub metadata: ProductMetadata,
// 
//     /// The number of risk states, i.e parameters tracking open positions in this outright product.
//     /// At 0, this means that no risk state is tracking positions(outstanding risk) in this outright product, which likely
//     /// means that there are no positions present in the outright product.
//     pub num_tracking_risk_states: usize,
// 
//     /// The status of the product.
//     pub product_status: ProductStatus,
// 
//     /// TODO
//     pub dust: Fractional,
// 
//     /// TODO
//     pub cumulative_funding_per_share: Fractional,
// 
//     /// TODO
//     pub cumulative_social_loss_per_share: Fractional,
// 
//     /// Open long positions opened on the exchange represented in notional value.
//     pub open_long_interest: Fractional,
// 
//     /// Open short positions opened on the exchange represented in notional value.
//     pub open_short_interest: Fractional,
// 
//     /// TODO
//     pub mark_price_qualifying_cum_value: Fractional,
// 
//     /// TODO
//     pub mark_price_max_qualifying_width: Fractional,
//     pub padding: [u64; 10],
// }
// 
// impl OutrightProduct {
//     /// [`apply_new_funding()`] allows us to update cumulative funding per share with new funding values(represented by `amt_per_share`),
//     /// followed by rounding it off to the number of decimals in cash.
//     pub fn apply_new_funding(
//         &mut self,
//         amt_per_share: Fractional,
//         cash_decimals: u64,
//     ) -> std::result::Result<(), DexError> {
//         self.cumulative_funding_per_share = (self.cumulative_funding_per_share + amt_per_share)
//             .round_unchecked(cash_decimals as u32)?;
// 
//         Ok(())
//     }
// 
//     /// [`apply_social_loss()`] allows us to update cumulative social loss per share with new social loss values(represented by `social_loss_per_share`)
//     /// followed by rounding it off to the number of decimals in cash.
//     pub fn apply_social_loss(
//         &mut self,
//         total_loss: Fractional,
//         total_shares: Fractional,
//         cash_decimals: u64,
//     ) -> std::result::Result<(), DexError> {
//         if total_shares > ZERO_FRAC {
//             let social_loss_per_share = total_loss.checked_div(total_shares)?;
// 
//             self.cumulative_social_loss_per_share +=
//                 social_loss_per_share.round_unchecked(cash_decimals as u32)?;
//         }
// 
//         Ok(())
//     }
// 
//     /// We determine that an outright product is dormant if there is no open long and short interest(i.e both equal to ZERO_FRAC)
//     pub fn is_dormant(&self) -> bool {
//         self.open_long_interest == ZERO_FRAC && self.open_short_interest == ZERO_FRAC
//     }
// 
//     /// We determine that an outright product is removable/settle-able if [`is_dormant()`] returns true AND there are no active risk states tracking the
//     /// product at the moment.
//     pub fn is_removable(&self) -> bool {
//         self.is_dormant() && self.num_tracking_risk_states == 0
//     }
// 
//     pub fn is_expired_or_expiring(&self) -> bool {
//         self.product_status == ProductStatus::Expiring
//             || self.product_status == ProductStatus::Expired
//     }
// 
//     pub fn is_expired(&self) -> bool {
//         self.product_status == ProductStatus::Expired
//     }
// 
//     pub fn is_expiring(&self) -> bool {
//         self.product_status == ProductStatus::Expiring
//     }
// 
//     pub fn is_unitialized(&self) -> bool {
//         self.product_status == ProductStatus::Uninitialized
//     }
// 
//     /// TODO: need to make sense of this
//     pub fn update_open_interest_change(
//         &mut self,
//         trade_size: Fractional,
//         buyer_short_position: Fractional,
//         seller_long_position: Fractional,
//     ) -> std::result::Result<(), DexError> {
//         match (
//             buyer_short_position < trade_size,
//             seller_long_position < trade_size,
//         ) {
//             (true, true) => {
//                 self.open_long_interest = self
//                     .open_long_interest
//                     .checked_add(trade_size)?
//                     .checked_sub(buyer_short_position)?
//                     .checked_sub(seller_long_position)?;
//             }
//             (true, false) => {
//                 self.open_long_interest =
//                     self.open_long_interest.checked_sub(buyer_short_position)?;
//             }
//             (false, true) => {
//                 self.open_long_interest =
//                     self.open_long_interest.checked_sub(seller_long_position)?;
//             }
//             (false, false) => {
//                 self.open_long_interest = self.open_long_interest.checked_sub(trade_size)?;
//             }
//         };
//         self.open_short_interest = self.open_long_interest;
//         Ok(())
//     }
// }
// 
// impl Deref for OutrightProduct {
//     type Target = ProductMetadata;
// 
//     fn deref(&self) -> &Self::Target {
//         &self.metadata
//     }
// }
// 
// impl DerefMut for OutrightProduct {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.metadata
//     }
// }
// 
// impl Default for OutrightProduct {
//     fn default() -> Self {
//         unsafe { std::mem::zeroed() }
//     }
// }
// 
// #[derive(
//     Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone, Copy,
// )]
// pub struct ComboProduct {
//     pub metadata: ProductMetadata,
//     pub num_legs: usize,
//     pub legs_array: [ComboLeg; MAX_LEGS],
// }
// 
// impl Default for ComboProduct {
//     fn default() -> Self {
//         unsafe { std::mem::zeroed() }
//     }
// }
// 
// impl ComboProduct {
//     pub fn legs(&self) -> &[ComboLeg] {
//         &self.legs_array[..self.num_legs]
//     }
// 
//     pub fn has_leg(&self, product_key: ProductId) -> bool {
//         self.legs()
//             .iter()
//             .take(self.num_legs)
//             .any(|l| l.product_key == product_key)
//     }
// 
//     // TODO: add MPG
//     // TODO: pub fn is_expired()
// 
//     /// [`get_combo_status`] returns the status of a combo product listed on an MPG.
//     pub fn get_combo_status(&self, mpg_min: &MarketProductGroupMin) -> ProductStatus {
//         // We first obtain the result of calling the `find_product_index_among_all` function, which returns
//         // the product id and the Product itself.
//         let result = mpg_min.find_product_index_among_all(&self.metadata.product_id);
// 
//         // If the result returns an `Ok()`, we return a block, otherwise we return
//         // [`ProductStatus::Uninitialized`]
//         match result {
//             Ok(_) => {}
//             Err(_) => return ProductStatus::Uninitialized, // if err, product doesn't exist
//         }
// 
//         // Then, we proceed to check if the legs of the combo product is initialized, or not.
//         // For that, we first construct the outright product from the [`ComboLeg`] struct, using the `find_outright_among_all` fn.
//         // Then, we return the product status. However, while constructing the outright, we do not find any outright,
//         // we return Uninitialized.
//         for leg in self.legs() {
//             if let Ok((_, outright)) = mpg_min.find_outright_among_all(&leg.product_key) {
//                 if outright.product_status != ProductStatus::Initialized {
//                     return outright.product_status;
//                 }
//             } else {
//                 return ProductStatus::Uninitialized;
//             }
//         }
//         ProductStatus::Initialized
//     }
// }
// 
// impl Deref for ComboProduct {
//     type Target = ProductMetadata;
// 
//     fn deref(&self) -> &Self::Target {
//         &self.metadata
//     }
// }
// 
// impl DerefMut for ComboProduct {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.metadata
//     }
// }
// 
// #[derive(
//     Debug,
//     Default,
//     Eq,
//     BorshSerialize,
//     BorshDeserialize,
//     PartialEq,
//     Serialize,
//     Deserialize,
//     Clone,
//     Copy,
// )]
// pub struct ComboLeg {
//     pub product_index: usize,
//     pub product_key: ProductId,
//     pub ratio: i64,
// }
// 
// #[derive(
//     Eq, Debug, PartialEq, Clone, Copy, BorshDeserialize, BorshSerialize, Deserialize, Serialize,
// )]
// #[repr(C, u64)]
// pub enum Product {
//     Outright { outright_product: OutrightProduct },
//     Combo { combo_product: ComboProduct },
// }
// 
// unsafe impl Zeroable for Product {}
// 
// impl Product {
//     pub fn get_best_bid(&self) -> Fractional {
//         match self {
//             Product::Outright { outright_product } => outright_product.metadata.prices.bid,
//             Product::Combo { combo_product } => combo_product.metadata.prices.bid,
//         }
//     }
// 
//     pub fn get_best_ask(&self) -> Fractional {
//         match self {
//             Product::Outright { outright_product } => outright_product.metadata.prices.ask,
//             Product::Combo { combo_product } => combo_product.metadata.prices.ask,
//         }
//     }
// 
//     pub fn get_prev_best_bid(&self, slot: u64) -> Fractional {
//         match self {
//             Product::Outright { outright_product } => {
//                 if slot > outright_product.metadata.prices.slot {
//                     outright_product.metadata.prices.bid
//                 } else {
//                     outright_product.metadata.prices.prev_bid
//                 }
//             }
//             Product::Combo { combo_product } => {
//                 if slot > combo_product.metadata.prices.slot {
//                     combo_product.metadata.prices.bid
//                 } else {
//                     combo_product.metadata.prices.prev_bid
//                 }
//             }
//         }
//     }
// 
//     pub fn get_prev_best_ask(&self, slot: u64) -> Fractional {
//         match self {
//             Product::Outright { outright_product } => {
//                 if slot > outright_product.metadata.prices.slot {
//                     outright_product.metadata.prices.ask
//                 } else {
//                     outright_product.metadata.prices.prev_ask
//                 }
//             }
//             Product::Combo { combo_product } => {
//                 if slot > combo_product.metadata.prices.slot {
//                     combo_product.metadata.prices.ask
//                 } else {
//                     combo_product.metadata.prices.prev_ask
//                 }
//             }
//         }
//     }
// 
//     pub fn try_to_combo(&self) -> std::result::Result<&ComboProduct, DexError> {
//         match self {
//             Product::Outright {
//                 outright_product: _,
//             } => Err(DexError::ProductNotCombo.into()),
//             Product::Combo { combo_product: c } => Ok(c),
//         }
//     }
// 
//     pub fn try_to_outright(&self) -> std::result::Result<&OutrightProduct, DexError> {
//         match self {
//             Product::Outright {
//                 outright_product: o,
//             } => Ok(o),
//             Product::Combo { combo_product: _ } => Err(DexError::ProductNotOutright.into()),
//         }
//     }
// 
//     pub fn try_to_combo_mut(&mut self) -> std::result::Result<&mut ComboProduct, DexError> {
//         match self {
//             Product::Outright {
//                 outright_product: _,
//             } => Err(DexError::ProductNotCombo.into()),
//             Product::Combo { combo_product: c } => Ok(c),
//         }
//     }
// 
//     pub fn try_to_outright_mut(&mut self) -> std::result::Result<&mut OutrightProduct, DexError> {
//         match self {
//             Product::Outright {
//                 outright_product: o,
//             } => Ok(o),
//             Product::Combo { combo_product: _ } => Err(DexError::ProductNotOutright.into()),
//         }
//     }
// 
//     pub fn get_ratios_and_product_indexes(
//         &self,
//         idx: usize,
//     ) -> impl Iterator<Item = (i64, usize)> + '_ {
//         match self {
//             Product::Outright {
//                 outright_product: _,
//             } => TwoIterators::A(([(1, idx)]).into_iter()),
//             Product::Combo { combo_product: c } => TwoIterators::B(
//                 c.legs()
//                     .iter()
//                     .take(c.num_legs)
//                     .map(|leg| (leg.ratio, leg.product_index)),
//             ),
//         }
//     }
// 
//     /// [`get_product_status`] returns the current product status listed on a Market Product Group.
//     /// If the product is an outright, we directly return the `product_status` field of outrights.
//     /// If the product is a combo product, we use the `get_combo_status` to obtain it's status.
//     pub fn get_product_status(&self, mpg_min: &MarketProductGroupMin) -> ProductStatus {
//         match self {
//             Product::Outright { outright_product } => outright_product.product_status,
//             Product::Combo { combo_product } => combo_product.get_combo_status(mpg_min),
//         }
//     }
// }
// 
// pub trait ProductTrait {
//     fn get_product_key(&self) -> &ProductId;
// 
//     fn is_combo(&self) -> bool;
// 
//     fn get_name(&self) -> &[u8; 16];
// 
//     fn get_orderbook_id(&self) -> &OrderbookId;
// }
// 
// impl ProductTrait for &Product {
//     #[inline]
//     fn get_product_key(&self) -> &ProductId {
//         match self {
//             Product::Outright { outright_product } => &outright_product.metadata.product_id,
//             Product::Combo { combo_product } => &combo_product.metadata.product_id,
//         }
//     }
// 
//     #[inline]
//     fn is_combo(&self) -> bool {
//         match self {
//             Product::Outright { outright_product } => false,
//             Product::Combo { combo_product } => true,
//         }
//     }
// 
//     #[inline]
//     fn get_name(&self) -> &[u8; 16] {
//         match self {
//             Product::Outright { outright_product } => &outright_product.metadata.name,
//             Product::Combo { combo_product } => &combo_product.metadata.name,
//         }
//     }
// 
//     #[inline]
//     fn get_orderbook_id(&self) -> &OrderbookId {
//         match self {
//             Product::Outright { outright_product } => &outright_product.metadata.orderbook_id,
//             Product::Combo { combo_product } => &combo_product.metadata.orderbook_id,
//         }
//     }
// }
// 
// impl ProductTrait for &OutrightProduct {
//     #[inline]
//     fn get_product_key(&self) -> &ProductId {
//         &self.metadata.product_id
//     }
// 
//     #[inline]
//     fn is_combo(&self) -> bool {
//         false
//     }
// 
//     #[inline]
//     fn get_name(&self) -> &[u8; 16] {
//         &self.metadata.name
//     }
// 
//     #[inline]
//     fn get_orderbook_id(&self) -> &OrderbookId {
//         &self.metadata.orderbook_id
//     }
// }
// 
// impl ProductTrait for &ComboProduct {
//     #[inline]
//     fn get_product_key(&self) -> &ProductId {
//         &self.metadata.product_id
//     }
// 
//     #[inline]
//     fn is_combo(&self) -> bool {
//         true
//     }
// 
//     #[inline]
//     fn get_name(&self) -> &[u8; 16] {
//         &self.metadata.name
//     }
// 
//     #[inline]
//     fn get_orderbook_id(&self) -> &OrderbookId {
//         &self.metadata.orderbook_id
//     }
// }
// 
// #[repr(transparent)]
// #[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
// pub struct ProductsArray {
//     pub array: [Product; MAX_PRODUCTS],
// }
// 
// impl Deref for ProductsArray {
//     type Target = [Product; MAX_PRODUCTS];
// 
//     fn deref(&self) -> &Self::Target {
//         &self.array
//     }
// }
// 
// impl DerefMut for ProductsArray {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.array
//     }
// }
