//! Implementation of mark prices in software.
//! Mark prices are a function of the index price, meaning that they are calculated as the index + (index-book EWMA).
//! Mark price calculation has other dependent variables, such as qualifying bid and ask prices, to ensure sufficiently low width,
//! and qualifying size, to ensure sufficiently high depth.

use {
    aaob::tree::{LeafNode, Slab},
    borsh::{BorshDeserialize, BorshSerialize},
    crate::{
        // address::ProductAddress,
        covariance_metadata::MAX_OUTRIGHTS,
        error::{ExchangeResult, RiskError},
        // fast_int::{FastInt, TWO_FAST_INT, ZERO_FAST_INT},
        // fractional::{Fractional, ZERO_FRAC},
        oracle_type::OracleType,
        temp::{
            mpg_minimal::MarketProductGroupMin,
            price_ewma::{NO_ASK, NO_BID},
            products::{ComboProduct, OutrightProduct, Product, ProductId},
        },
    },
    serde::{self, Deserialize, Serialize},
    sov_modules_api::Spec,
    spicenet_shared::fast_int::{FastInt, TWO_FAST_INT, ZERO_FAST_INT},
    spicenet_shared::fractional::{Fractional, ZERO_FRAC},
    std::ops::{Deref, DerefMut},
};

pub const MAX_MARK_PRICES: usize = MAX_OUTRIGHTS / 2;

fn lossy_div(a: Fractional, b: Fractional) -> Fractional {
    let a: FastInt = a.into();
    let b: FastInt = b.into();

    (a / b).to_frac().unwrap()
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MarkPricesArray {
    pub is_default_oracle_used: bool,
    pub oracle_type: OracleType,
    pub oracle_id: OracleId,
    pub array: [MarkPrice; MAX_MARK_PRICES],
}

#[derive(
    Copy, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize, Eq, PartialEq, Debug,
)]
pub struct OracleId(u64);

impl OracleId {
    // never use this
    pub fn new(&self) -> Self {
        OracleId(0)
    }

    // always use this
    pub fn new_with_id(&self, id: u64) -> Self {
        OracleId(id)
    }
}

impl Deref for MarkPricesArray {
    type Target = [MarkPrice; MAX_MARK_PRICES];

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl DerefMut for MarkPricesArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

#[derive(
    Eq, Debug, PartialEq, Clone, Copy, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
pub struct MarkPrice {
    pub product_addy: ProductId,
    pub mark_price: FastInt,
    pub prev_oracle_minus_book_ewma: FastInt,
    pub oracle_minus_book_ewma: FastInt,
    pub last_update_slot: Option<u64>,
    pub qualifying_bid_price: Option<Fractional>,
    pub qualifying_ask_price: Option<Fractional>,
}

impl MarkPrice {
    pub const SIZE: usize = std::mem::size_of::<MarkPrice>();
}

impl Default for MarkPrice {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl MarkPricesArray {
    pub const SIZE: usize = std::mem::size_of::<MarkPricesArray>();
    pub const MAX_BOOK_SPREAD_FOR_VALID_MARK_PRICE: FastInt = FastInt { value: 30_000_i128 };

    pub fn calculate_price(
        &self,
        mpg_min: &MarketProductGroupMin,
        product: &Product,
        curr_time: u64,
    ) -> ExchangeResult<FastInt> {
        match product {
            Product::Outright { outright_product } => {
                self.calculate_outright_mp(outright_product, curr_time)
            }
            Product::Combo { combo_product } => {
                self.calculate_combo_mp(mpg_min, combo_product, curr_time)
            }
        }
    }

    pub fn calculate_combo_mp(
        &self,
        mpg_min: &MarketProductGroupMin,
        combo: &ComboProduct,
        curr_time: u64,
    ) -> ExchangeResult<FastInt> {
        let mut price = ZERO_FAST_INT;
        for leg_idx in 0..combo.num_legs {
            let leg = &combo.legs()[leg_idx];
            let market_product_idx = leg.product_index;
            let product = &mpg_min.active_products[market_product_idx];
            let outright_mp = match product { // TODO (@karthik-pepperdex): fix
                Product::Outright { outright_product } => {
                    // making sure that the product that the leg refers to is the same product the leg thinks it is referring to.
                    assert_eq!(leg.product_key, outright_product.product_id);
            
                    self.calculate_outright_mp(&outright_product, curr_time)?
                }
                _ => return Err(RiskError::UnexpectedProductType.into()),
            };
            
            price += leg.ratio * outright_mp;
        }

        Ok(price)
    }

    /// Calculates the mark price of an outright product.
    /// First, we obtain the product id using the `get_product_index` fn.
    /// Then, we find the last update slot of the mark price and break the logic if the mark price is too stale
    /// Otherwise, we just return the mark price from the mark price array using the id.
    pub fn calculate_outright_mp(
        &self,
        outright: &OutrightProduct,
        curr_time: u64,
    ) -> ExchangeResult<FastInt> {
        let idx = self.get_product_index(&outright.product_id)?;
        let last_update_time = self.array[idx].last_update_slot;
        if last_update_time.unwrap() + 15 < curr_time {
            return Err(RiskError::MarkPricesOutOfDate.into());
        }

        Ok(self.array[idx].mark_price)
    }

    /// Returns the product index upon providing the ProductId.
    /// We iterate through the MarkPricesArray and find the mark price whose id matches with the id provided, and once found,
    /// we return the same.
    pub fn get_product_index(&self, product_id: &ProductId) -> ExchangeResult<usize> {
        match self
            .array
            .iter()
            .enumerate()
            .find(|(_, mark_price)| mark_price.product_addy == *product_id)
        {
            Some((i, _)) => Ok(i),
            _ => Err(RiskError::MissingMarkPrice.into()),
        }
    }

    fn calculate_qualifying_market_mid(
        &self,
        qualifying_best_bid: Option<Fractional>,
        qualifying_best_ask: Option<Fractional>,
        index_price: Fractional,
        max_qualifying_width: Fractional,
    ) -> Option<FastInt> {
        if qualifying_best_bid.is_none()
            || qualifying_best_ask.is_none()
            || lossy_div(
                qualifying_best_ask.unwrap() - qualifying_best_bid.unwrap(),
                index_price,
            ) > max_qualifying_width
        {
            None
        } else {
            Some(
                FastInt::from(qualifying_best_ask.unwrap() + qualifying_best_bid.unwrap())
                    / TWO_FAST_INT,
            )
        }
    }

    fn calculate_weighted_best_price<I>(
        &self,
        outright: &OutrightProduct,
        slab_iter: I,
        qualifying_cumulative_qty: Fractional,
    ) -> Option<Fractional>
    where
        I: Iterator<Item = LeafNode>,
    {
        let mut cum_weighted_price = ZERO_FRAC;
        let mut cum_qty = ZERO_FRAC;

        for node in slab_iter {
            let price = Fractional::new((node.price() >> 32) as i64, 0)
                * outright.metadata.tick_size
                - outright.metadata.price_offset;

            let qty = Fractional::new(node.base_qty as i64, outright.base_decimals);

            if cum_qty + qty > qualifying_cumulative_qty {
                let remaining_qty = qualifying_cumulative_qty - cum_qty;
                cum_weighted_price += price * remaining_qty;
                cum_qty += remaining_qty;
                break;
            } else {
                cum_qty += qty;
                cum_weighted_price += price * qty;
            }
        }

        if cum_qty < qualifying_cumulative_qty {
            return None;
        }

        Some(lossy_div(cum_weighted_price, cum_qty))
    }

    pub fn calculate_outright_book_price(
        &self,
        outright: &OutrightProduct,
    ) -> ExchangeResult<FastInt> {
        let prev_bid = outright.prices.prev_bid;
        let prev_ask = outright.prices.prev_ask;

        let (price, spread) = match (prev_bid > NO_BID, prev_ask < NO_ASK) {
            (true, true) => (
                FastInt::from(prev_bid + prev_ask) / TWO_FAST_INT,
                FastInt::from(prev_ask.checked_sub(prev_bid)?),
            ),
            _ => {
                let bid = outright.prices.bid;
                let ask = outright.prices.ask;

                match (bid > NO_BID, ask < NO_ASK) {
                    (true, true) => (
                        FastInt::from(ask + bid) / TWO_FAST_INT,
                        FastInt::from(ask.checked_sub(bid)?),
                    ),
                    _ => return Err(RiskError::MissingBBOForMarkPrice.into()),
                }
            }
        };

        if spread.abs() > (price * MarkPricesArray::MAX_BOOK_SPREAD_FOR_VALID_MARK_PRICE).abs() {
            Err(RiskError::BookSpreadTooWideForMarkPrice.into())
        } else {
            Ok(price)
        }
    }

    pub fn calculate_outright_book_price_with_slab<'a, S: Spec>(
        &mut self,
        outright: &OutrightProduct,
        index_price: Fractional,
        product_idx: usize,
        bids: Slab<'a>,
        asks: Slab<'a>,
    ) -> Option<FastInt>
    where
        S: Spec,
    {
        if outright.mark_price_qualifying_cum_value == ZERO_FRAC {
            return None;
        }

        let qualifying_cum_qty = lossy_div(outright.mark_price_qualifying_cum_value, index_price);

        let qualifying_bid_price =
            self.calculate_weighted_best_price(outright, bids.into_iter(false), qualifying_cum_qty);
        let qualifying_ask_price =
            self.calculate_weighted_best_price(outright, asks.into_iter(true), qualifying_cum_qty);

        self.array[product_idx].qualifying_bid_price = qualifying_bid_price;
        self.array[product_idx].qualifying_ask_price = qualifying_ask_price;

        let price = self.calculate_qualifying_market_mid(
            qualifying_bid_price,
            qualifying_ask_price,
            index_price,
            outright.mark_price_max_qualifying_width,
        );

        price.map(|price| FastInt::from(price))
    }

    pub fn calculate_outright_price_with_slab<'a, S>(
        &mut self,
        outright: &OutrightProduct,
        index_price: FastInt,
        product_idx: usize,
        curr_time: u64,
        bids: Slab<'a>,
        asks: Slab<'a>,
    ) -> ExchangeResult<FastInt>
    where
        S: Spec,
    {
        let book_price = self.calculate_outright_book_price_with_slab(
            outright,
            index_price.to_frac().unwrap(),
            product_idx,
            bids,
            asks,
        );
        self.update_mark_prices(book_price, index_price, product_idx, curr_time)
    }

    pub fn initialize_outright_mp(
        &mut self,
        outright: &OutrightProduct,
        index_price: FastInt,
    ) -> ExchangeResult<usize> {
        // im not totally sure of the logic of finding the product idx, specifically the `find` method.
        let product_idx = match self
            .array
            .iter()
            .enumerate()
            .find(|(_, mp)| mp.product_addy == ProductId::new(&ProductId(0)))
        {
            Some((i, _)) => i,
            _ => {
                return Err(RiskError::MarkPricesArrayIsFull.into());
            }
        };

        self.array[product_idx] = MarkPrice {
            product_addy: *outright.product_id,
            mark_price: index_price,
            prev_oracle_minus_book_ewma: ZERO_FAST_INT,
            oracle_minus_book_ewma: ZERO_FAST_INT,
            last_update_slot: None, //
            qualifying_ask_price: None,
            qualifying_bid_price: None,
        };

        Ok(product_idx)
    }

    pub fn collect_garbage(
        &mut self,
        mpg_min: &MarketProductGroupMin,
        max_products_to_examine: usize,
    ) -> ExchangeResult<()> {
        let mut num_entries_cleared = 0;
        let mut num_products_examined = 0;

        for i in 0..self.array.len() {
            // block
            {
                let mp = &self.array[i];
                // im not totally sure of this condition
                if mp.product_addy == ProductId::new(&ProductId(0)) {
                    continue;
                }

                num_products_examined += 1;
                if num_products_examined > max_products_to_examine {
                    break;
                }

                if mpg_min.find_product_index(&mp.product_addy).is_ok() {
                    continue;
                }

                num_entries_cleared += 1;
            }

            self.array[i] = MarkPrice {
                product_addy: ProductId::new(ProductId),
                mark_price: ZERO_FAST_INT,
                prev_oracle_minus_book_ewma: ZERO_FAST_INT,
                oracle_minus_book_ewma: ZERO_FAST_INT,
                last_update_slot: 0,
                qualifying_ask_price: None,
                qualifying_bid_price: None,
            };
        }

        Ok(())
    }

    fn update_mark_prices(
        &mut self,
        book_price: Option<FastInt>,
        index_price: FastInt,
        product_idx: usize,
        curr_time: u64,
    ) -> ExchangeResult<FastInt> {
        /// EMA notes
        /// We want to come up with a statistic s_t associated to current time t where t is part of the time series
        /// of book prices x_0, x_1 and so on.
        ///
        /// s_t = alpha * x_t + (1 - alpha) * s_[t-1] = s_[t-1] + alpha(x_t - s_[t-1]), thereby deriving s_t with the help of alpha.
        ///
        /// Alpha here is our smoothing factor and has to be in between 0 and 1. so 0 <= alpha <=1
        const ALPHA: FastInt = FastInt {
            value: 100_000_i128,
        };

        if self.array[product_idx].last_update_slot.unwrap() < curr_time {
            // this condition prevents exponentiation of the EMA and preserves the property that all mark price calculations use the recent book price only.
            // this is done by storing the current ema value so repeated calls to update within the same slot will use *that* value to calculate the new one.
            self.array[product_idx].prev_oracle_minus_book_ewma =
                self.array[product_idx].oracle_minus_book_ewma;
        }

        let prev_s_t = self.array[product_idx].prev_oracle_minus_book_ewma;
        let s_t = match book_price {
            Some(book_p) => prev_s_t + ALPHA.mul_zero_okay((index_price - book_p) - prev_s_t),
            None => prev_s_t,
        };

        self.array[product_idx].oracle_minus_book_ewma = s_t;
        self.array[product_idx].mark_price = index_price - s_t;
        self.array[product_idx].last_update_slot = Option::from(curr_time);

        Ok(self.array[product_idx].mark_price)
    }
}
