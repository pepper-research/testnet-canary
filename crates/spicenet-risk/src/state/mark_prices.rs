use std::ops::{Deref, DerefMut};

use sokoban::critbit::CritbitIterator;
use sov_modules_api::{Address, Spec};
use spicenet_aaob::{Order, Slab, MAX_SIZE, NUM_NODES};
use spicenet_shared::dex::{ComboProduct, OutrightProduct};
use spicenet_shared::risk::RiskError;
use spicenet_shared::time::Slot;
use spicenet_shared::{
    FastInt, Fractional, ProductId, NO_ASK_PRICE, NO_BID_PRICE, TWO_FAST_INT, ZERO_FAST_INT,
    ZERO_FRAC,
};
use spicenet_shared::{MarketProductGroup, Product};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct MarkPricesArray<S: Spec> {
    pub hardcoded_oracle_id: Option<Address<S>>, // TODO(!oracle): change to something like OracleId once oracle is done
    // array: [MarkPrice; MAX_MARK_PRICES],
    pub array: Vec<MarkPrice>,
}

impl<S: Spec> Deref for MarkPricesArray<S> {
    // type Target = [MarkPrice; MAX_MARK_PRICES];
    type Target = Vec<MarkPrice>;

    fn deref(&self) -> &Self::Target {
        &self.array
    }
}

impl<S: Spec> DerefMut for MarkPricesArray<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array
    }
}

fn lossy_div(a: Fractional, b: Fractional) -> Fractional {
    let a: FastInt = a.into();
    let b: FastInt = b.into();

    (a / b).to_frac().unwrap()
}

impl<S: Spec> MarkPricesArray<S> {
    pub const MAX_BOOK_SPREAD_FOR_VALID_MARK_PRICE: FastInt = FastInt { value: 30_000_i128 };

    pub fn calculate_price(
        &self,
        mpg_min: &MarketProductGroup<S>,
        product: &Product,
        curr_time: u64,
    ) -> Result<FastInt, RiskError> {
        match product {
            Product::Outright { outright_product } => {
                self.get_outright_price(outright_product, curr_time)
            }
            Product::Combo { combo_product } => {
                self.calculate_combo_price(mpg_min, combo_product, curr_time)
            }
        }
    }

    pub fn calculate_combo_price(
        &self,
        market_product_group: &MarketProductGroup<S>,
        combo: &ComboProduct,
        current_slot: Slot,
    ) -> Result<FastInt, RiskError> {
        let mut price = ZERO_FAST_INT;
        for leg_index in 0..combo.num_legs {
            let leg = &combo.legs()[leg_index];
            let market_product_index = leg.product_index;
            let product = &market_product_group.active_products.array[market_product_index];

            price += leg.ratio
                * match product {
                    Product::Outright { outright_product } => {
                        // for sanity, make sure that the product that the leg refers to
                        // is the same product the leg thinks it is referring to
                        assert_eq!(leg.product_key, outright_product.product_id,);

                        self.get_outright_price(&outright_product, current_slot)?
                    }
                    _ => return Err(RiskError::UnexpectedProductType.into()),
                };
        }

        Ok(price)
    }

    pub fn get_outright_price(
        &self,
        outright: &OutrightProduct,
        current_slot: Slot,
    ) -> Result<FastInt, RiskError> {
        let index = self.get_product_index(&outright.product_id)?;
        let update_slot = self.array[index].update_slot;
        if update_slot + 15 < current_slot {
            // msg!(
            //     "error: mark prices out of date. (update_slot {} current_slot {})",
            //     update_slot,
            //     current_slot,
            // );
            return Err(RiskError::MarkPricesOutOfDate.into());
        }
        Ok(self.array[index].mark_price)
    }

    pub fn get_product_index(&self, product_id: &ProductId) -> Result<usize, RiskError> {
        // msg!("literally trying to find {}", &product_key);
        match self
            .array
            .iter()
            .enumerate()
            .find(|(_, &mp)| mp.product_id == *product_id)
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

    fn calculate_weighted_best_price(
        &self,
        outright: &OutrightProduct,
        slab_iter: CritbitIterator<Order, NUM_NODES, MAX_SIZE>,
        qualifying_cum_qty: Fractional,
    ) -> Option<Fractional> {
        //TODO(!aaob, @sahilpabale): cross check the logic to make sure it is compatible with our tree implementation
        let mut cum_weighted_price = ZERO_FRAC;
        let mut cum_qty = ZERO_FRAC;

        for (_, node) in slab_iter {
            let price = Fractional::new((node.price >> 32) as i64, 0) * outright.metadata.tick_size
                - outright.metadata.price_offset;
            let qty = Fractional::new(node.base_qty as i64, outright.base_decimals);

            if cum_qty + qty > qualifying_cum_qty {
                let remaining_qty = qualifying_cum_qty - cum_qty;
                cum_weighted_price += price * remaining_qty;
                cum_qty += remaining_qty;
                break;
            } else {
                cum_qty += qty;
                cum_weighted_price += price * qty;
            }
        }

        if cum_qty < qualifying_cum_qty {
            return None;
        }

        Some(lossy_div(cum_weighted_price, cum_qty))
    }

    pub fn calculate_outright_book_price_with_slab(
        &mut self,
        outright: &OutrightProduct,
        index_price: Fractional,
        product_index: usize,
        bids: Slab, // TODO(!aaob): type depends on aaob
        asks: Slab, // TODO(!aaob): type depends on aaob
    ) -> Option<FastInt> {
        //TODO(!aaob, @sahilpabale): cross check the logic to make sure it is compatible with our tree implementation
        if outright.mark_price_qualifying_cum_value == ZERO_FRAC {
            return None;
        }

        let qualifying_cum_qty = lossy_div(outright.mark_price_qualifying_cum_value, index_price);

        let qualifying_bid_price =
            self.calculate_weighted_best_price(outright, bids.tree.into_iter(), qualifying_cum_qty);
        let qualifying_ask_price =
            self.calculate_weighted_best_price(outright, asks.tree.into_iter(), qualifying_cum_qty);

        self.array[product_index].qualifying_bid_price = Some(qualifying_bid_price?);
        self.array[product_index].qualifying_ask_price = Some(qualifying_ask_price?);

        let price = self.calculate_qualifying_market_mid(
            qualifying_bid_price,
            qualifying_ask_price,
            index_price,
            outright.mark_price_max_qualifying_width,
        );

        price.map(|price| FastInt::from(price))
    }

    pub fn calculate_outright_book_price(
        &self,
        outright: &OutrightProduct,
    ) -> Result<FastInt, RiskError> {
        let prev_bid = outright.prices.prev_bid;
        let prev_ask = outright.prices.prev_ask;

        let (price, spread) = match (prev_bid > NO_BID_PRICE, prev_ask < NO_ASK_PRICE) {
            // there exists both a previous bid and ask, take the midpoint
            (true, true) => (
                FastInt::from(prev_bid + prev_ask) / TWO_FAST_INT,
                FastInt::from(prev_ask.checked_sub(prev_bid).unwrap()),
            ),
            _ => {
                let bid = outright.prices.bid;
                let ask = outright.prices.ask;
                // look at the bid and ask from the market product
                match (bid > NO_BID_PRICE, ask < NO_ASK_PRICE) {
                    // there exists both a bid and ask, take the midpoint
                    (true, true) => (
                        FastInt::from(bid + ask) / TWO_FAST_INT,
                        FastInt::from(ask.checked_sub(bid).unwrap()),
                    ),
                    _ => return Err(RiskError::MissingBBOForMarkPrice.into()),
                }
            }
        };

        if spread.abs() > (price * MarkPricesArray::<S>::MAX_BOOK_SPREAD_FOR_VALID_MARK_PRICE).abs()
        {
            Err(RiskError::BookSpreadTooWideForMarkPrice.into())
        } else {
            Ok(price)
        }
    }

    pub fn initialize_outright_price(
        &mut self,
        outright: &OutrightProduct,
        index_price: FastInt,
    ) -> Result<usize, RiskError> {
        // // TODO replace with bitset
        // let product_index = match self
        //     .array
        //     .iter()
        //     .enumerate()
        //     .find(|(_, &mp)| mp.product_key == system_program::ID) // system_program::ID means zero'd out
        // {
        //     Some((i, _)) => i,
        //     _ => { return Err(RiskError::MarkPricesArrayIsFull.into()); }
        // };

        // get the next available index
        let product_index = self.array.len();

        self.array[product_index] = MarkPrice {
            // product_key: outright.product_key,
            product_id: outright.product_id,
            mark_price: index_price,
            prev_oracle_minus_book_ewma: ZERO_FAST_INT,
            oracle_minus_book_ewma: ZERO_FAST_INT,
            update_slot: 0,
            qualifying_bid_price: None,
            qualifying_ask_price: None,
        };
        Ok(product_index)
    }

    pub fn collect_garbage(
        &mut self,
        mpg: &MarketProductGroup<S>,
        max_products_to_examine: usize,
    ) -> Result<(), RiskError> {
        // msg!("collecting garbage; clearing mp array at indices:");
        let mut num_entries_cleared = 0;
        let mut num_products_examined = 0;
        for i in 0..self.array.len() {
            {
                let mp = &self.array[i];
                // TODO: get back to this but we typically expect the products to be initialized
                // if mp.product_key == system_program::ID {
                //     continue;
                // }
                num_products_examined += 1;
                if num_products_examined > max_products_to_examine {
                    break;
                }
                if mpg.find_product_index(&mp.product_id).is_some() {
                    continue;
                }
                num_entries_cleared += 1;
            }
            // msg!("{}", i);
            self.array[i] = MarkPrice {
                // product_key: system_program::ID,
                product_id: ProductId::from_const_slice(Default::default()), // TODO: figure out a better default for empty mark prices
                mark_price: ZERO_FAST_INT,
                prev_oracle_minus_book_ewma: ZERO_FAST_INT,
                oracle_minus_book_ewma: ZERO_FAST_INT,
                update_slot: 0,
                qualifying_bid_price: None,
                qualifying_ask_price: None,
            };
        }
        // msg!("total cleared: {}", num_entries_cleared);
        Ok(())
    }

    fn update_mark_price(
        &mut self,
        book_price: Option<FastInt>,
        index_price: FastInt,
        product_index: usize,
        current_slot: Slot,
    ) -> Result<FastInt, RiskError> {
        // EMA formula:
        // We want to come up with the statistic s_t associated to time t
        // of the time series of book prices x_0, x_1, x_2, ...
        //
        // s_t = alpha * x_t + (1 - alpha) * s_[t-1]
        //     = s_[t-1] + alpha*(x_t - s_[t-1])
        //
        // where alpha is the "smoothing factor" and 0 <= alpha <= 1
        const ALPHA: FastInt = FastInt {
            value: 100_000_i128, // ALPHA = 0.1
        };
        if self.array[product_index].update_slot < current_slot {
            // This if statement basically says: if we haven't yet updated *this slot*, then
            // store the current EMA value so repeated calls to update within one slot will
            // only use the current EMA value in the calculation of the new one.
            // This prevents "exponentiation" of the EMA while preserving the property that
            // the calculated mark price always uses the most recent book price.
            self.array[product_index].prev_oracle_minus_book_ewma =
                self.array[product_index].oracle_minus_book_ewma;
        };
        let prev_s_t = self.array[product_index].prev_oracle_minus_book_ewma;
        let s_t = match book_price {
            Some(book_px) => prev_s_t + ALPHA.mul_zero_okay((index_price - book_px) - prev_s_t),
            None => prev_s_t,
        };
        self.array[product_index].oracle_minus_book_ewma = s_t;
        self.array[product_index].mark_price = index_price - s_t;
        self.array[product_index].update_slot = current_slot;
        // msg!(
        //     "index: {}; book: {}; s_t: {}; mark: {}",
        //     index_price,
        //     book_price,
        //     self.array[product_index].oracle_minus_book_ewma,
        //     self.array[product_index].mark_price
        // );
        Ok(self.array[product_index].mark_price)
    }

    pub fn update_outright_price_with_slab(
        &mut self,
        outright: &OutrightProduct,
        index_price: FastInt,
        product_index: usize,
        current_slot: Slot,
        bids: Slab, // TODO(!aaob): type depends on aaob
        asks: Slab, // TODO(!aaob): type depends on aaob
    ) -> Result<FastInt, RiskError> {
        //TODO(!aaob, @sahilpabale): cross check the logic to make sure it is compatible with our tree implementation
        let book_price = self.calculate_outright_book_price_with_slab(
            outright,
            index_price.to_frac().unwrap(),
            product_index,
            bids,
            asks,
        );
        if let Some(px) = book_price {
            // msg!(
            //     "Successfully calculated qualifying price for product {} from the book -> {}",
            //     outright.product_key,
            //     px
            // );
        }
        self.update_mark_price(book_price, index_price, product_index, current_slot)
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Copy)]
pub struct MarkPrice {
    pub product_id: ProductId,
    pub mark_price: FastInt,
    pub prev_oracle_minus_book_ewma: FastInt,
    pub oracle_minus_book_ewma: FastInt,
    pub update_slot: Slot,
    pub qualifying_bid_price: Option<Fractional>,
    pub qualifying_ask_price: Option<Fractional>,
}
