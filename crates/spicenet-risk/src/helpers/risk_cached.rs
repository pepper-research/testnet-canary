use anyhow::Result;
use sov_modules_api::Spec;

use spicenet_shared::dex::{MarketProductGroup, ProductTrait, TraderRiskGroup};
use spicenet_shared::time::Slot;
use spicenet_shared::{
    ComboProduct, FastInt, Fractional, IsInitialized, NEGATIVE_ERROR_ROUND_TO_ZERO_FAST_INT,
    TWO_FAST_INT, ZERO_FAST_INT, ZERO_FRAC,
};

use crate::RiskModule;
use crate::{CovarianceMatrix, MarkPricesArray, RiskProfile};
use spicenet_shared::risk::health_status::HealthStatus;
use spicenet_shared::risk::{
    ActionStatus, LiquidationStatus, SocialLossInfo, VarianceCache, MAX_TRADER_POSITIONS,
};
use spicenet_shared::risk::{RiskError, MAX_OUTRIGHTS};

struct TraderPositionMetadata {
    // the index of this product in the products array
    product_index: usize,
    // the change in position vs. the existing variance cache
    position_change: FastInt,
    // total bid quantity in open orders
    open_bid_qty: FastInt,
    // total ask quantity in open orders
    open_ask_qty: FastInt,
    // the index of this product's data in our variance cache
    variance_cache_index: Option<usize>,
}

const REBUILD_CACHE_CASE: RiskError = RiskError::FastIntCoercionToZero;

impl<S: Spec> RiskModule<S> {
    pub fn calculate_risk_profile_cached(
        market_product_group: &MarketProductGroup<S>,
        mark_prices: &MarkPricesArray<S>,
        trader_risk_group: &TraderRiskGroup<S>,
        covariance_matrix: &CovarianceMatrix,
        cache: &mut VarianceCache,
        slot: Slot,
    ) -> Result<RiskProfile> {
        // let mut abs_position_value = [ZERO_FAST_INT; MAX_TRADER_POSITIONS];
        let mut abs_position_value = Vec::with_capacity(MAX_TRADER_POSITIONS);
        let mut total_abs_position_value = ZERO_FAST_INT;

        let is_force_rebuild =
            if true || (covariance_matrix.covariance_metadata.update_slot > cache.update_offset) {
                // if the covariance matrix has updated since our last cache update, we have to rebuild the entire thing
                // msg!("risk clock.slot: {}", slot);
                // msg!("Calculating risk and rebuilding trader cache");
                Self::calculate_risk_rebuild_cache(
                    market_product_group,
                    mark_prices,
                    trader_risk_group,
                    covariance_matrix,
                    cache,
                    &mut abs_position_value,
                    &mut total_abs_position_value,
                    slot,
                )?;
                false // do not force rebuild a second time
            } else {
                // msg!("Calculating risk using existing trader cache");
                match Self::calculate_risk_from_cache(
                    market_product_group,
                    mark_prices,
                    trader_risk_group,
                    covariance_matrix,
                    cache,
                    &mut abs_position_value,
                    &mut total_abs_position_value,
                    slot,
                ) {
                    Ok(_) => false, // do not force rebuild, calculating risk from cache succeeded
                    Err(e) => {
                        if e.is::<RiskError>()
                            && e.downcast_ref::<RiskError>() == Some(&REBUILD_CACHE_CASE)
                        {
                            true // fully return from the outer function with the error
                        } else {
                            return Err(e);
                        }
                    }
                }
            };

        if is_force_rebuild
            || (cache.total_variance_traded < ZERO_FAST_INT
                && cache.total_variance_traded >= NEGATIVE_ERROR_ROUND_TO_ZERO_FAST_INT)
        {
            // msg!("Small negative variance --> rebuilding trader cache");
            Self::calculate_risk_rebuild_cache(
                market_product_group,
                mark_prices,
                trader_risk_group,
                covariance_matrix,
                cache,
                &mut abs_position_value,
                &mut total_abs_position_value,
                slot,
            )?;
        }

        cache.update_offset = slot;

        // msg!(
        //     "Total Variance: {}, Open Order Variance: {}, Position Value: {}",
        //     cache.total_variance,
        //     cache.open_order_variance,
        //     cache.position_value
        // );

        let mut risk_profile: RiskProfile = (&*cache, trader_risk_group).into();
        risk_profile.abs_position_value = abs_position_value;
        risk_profile.total_abs_position_value = total_abs_position_value;
        Ok(risk_profile)
    }

    pub fn calculate_risk_rebuild_cache(
        market_product_group: &MarketProductGroup<S>,
        mark_prices: &MarkPricesArray<S>,
        trader_risk_group: &TraderRiskGroup<S>,
        covariance_matrix: &CovarianceMatrix,
        cache: &mut VarianceCache,
        // abs_position_value: &mut [FastInt; MAX_TRADER_POSITIONS],
        abs_position_value: &mut Vec<FastInt>,
        total_abs_position_value: &mut FastInt,
        current_slot: Slot,
    ) -> Result<()> {
        let mut q: Vec<TraderPositionMetadata> = Vec::with_capacity(MAX_TRADER_POSITIONS);

        cache.total_variance_traded = ZERO_FAST_INT;
        cache.derivative_position_value = ZERO_FAST_INT;
        cache.total_liquidity_buffer = FastInt::from(-1);

        let mut liquidity_buffer_usd: Option<FastInt> = Some(FastInt::from(0));

        for (trader_position_index, trader_position) in
            trader_risk_group.trader_positions.iter().enumerate()
        {
            if !trader_position.is_initialized() {
                continue;
            }

            let product_index: usize = trader_position.product_index;
            let base_decimals =
                market_product_group.active_products.array[product_index].base_decimals; // TODO: figure this out
            let position: FastInt =
                (trader_position.position + trader_position.pending_position).into();
            let (open_bid_qty, open_ask_qty) = {
                let open_bid_qty_book: FastInt = Fractional::new(
                    trader_risk_group.open_orders.products[product_index].bid_qty_in_book,
                    base_decimals,
                )
                .into();
                let open_ask_qty_book: FastInt = Fractional::new(
                    trader_risk_group.open_orders.products[product_index].ask_qty_in_book,
                    base_decimals,
                )
                .into();
                let open_bid_qty_lock: FastInt = trader_risk_group.locked_collateral
                    [trader_position_index]
                    .bid_qty
                    .into();
                let open_ask_qty_lock: FastInt = trader_risk_group.locked_collateral
                    [trader_position_index]
                    .ask_qty
                    .into();
                (
                    open_bid_qty_book + open_bid_qty_lock,
                    open_ask_qty_book + open_ask_qty_lock,
                )
            };

            // only continue on if there is a position change or existing open orders
            // THE ABOVE COMMENT IS NO LONGER TRUE^ because even if no bid, no ask, and no position, this product could be a leg of a combo that DOES have an open bid or open ask...
            // ... therefore, the following if statement is commented out.
            // if position == ZERO_FAST_INT && open_bid_qty.m == 0 && open_ask_qty.m == 0 {
            //     continue;
            // }

            q.push(TraderPositionMetadata {
                product_index,
                position_change: position,
                open_bid_qty,
                open_ask_qty,
                // the order of the elements in the vector will be the order that they are added to the variance cache
                variance_cache_index: Some(q.len()),
            });

            if position != ZERO_FAST_INT {
                // calculate price of this derivative
                let product = &market_product_group.active_products[product_index];
                let price =
                    mark_prices.calculate_price(market_product_group, product, current_slot)?;

                let this_position_value = price.mul_clamp_to_tick(position);
                let this_position_value_abs = this_position_value.abs();

                // accumulate total derivative position value
                cache.derivative_position_value += this_position_value;

                // store the abs(position_value) at the product_index, and accumulate the total
                abs_position_value[product_index] = this_position_value_abs;
                *total_abs_position_value += this_position_value_abs;

                if !product.is_combo() {
                    match liquidity_buffer_usd {
                        Some(ref mut buffer) => {
                            let mp_product_index =
                                mark_prices.get_product_index(&product.product_id)?; // TODO: figure this out
                            let mark_price = price.to_frac().unwrap();

                            let (is_long, qualifying_price): (bool, Option<Fractional>) =
                                if position > ZERO_FAST_INT {
                                    (
                                        true,
                                        mark_prices.array[mp_product_index].qualifying_bid_price,
                                    )
                                } else {
                                    (
                                        false,
                                        mark_prices.array[mp_product_index].qualifying_ask_price,
                                    )
                                };

                            let px = match qualifying_price {
                                Some(q_px) => Some(q_px),
                                None => {
                                    let outright = product.try_to_outright().unwrap();
                                    if outright.mark_price_qualifying_cum_value == ZERO_FRAC {
                                        match mark_prices.calculate_outright_book_price(outright) {
                                            Ok(px) => Some(px.to_frac().unwrap()),
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    }
                                }
                            };

                            match px {
                                Some(book_price) => {
                                    if (is_long && mark_price > book_price)
                                        || (!is_long && mark_price < book_price)
                                    {
                                        *buffer += FastInt::from(
                                            (position.to_frac().unwrap()
                                                * (mark_price - book_price))
                                                .abs(),
                                        );
                                    }
                                }
                                None => {
                                    liquidity_buffer_usd = None;
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        }

        if !liquidity_buffer_usd.is_none() {
            cache.total_liquidity_buffer = liquidity_buffer_usd.unwrap();
        }

        for tpm1 in q.iter() {
            // this will accumulate sigma * q for product_index1
            let mut variance_term = ZERO_FAST_INT;
            let variance_cache_index = tpm1.variance_cache_index.unwrap();

            for tpm2 in q.iter() {
                // TODO: is there an elegant way to cache this covariance lookup, because we will
                // need it again in this loop
                let covariance = covariance_matrix
                    .get_covariance_from_product_indexes(tpm1.product_index, tpm2.product_index)?;

                variance_term += covariance * tpm2.position_change;
            }

            // accumulate the total variance
            cache.total_variance_traded += variance_term * tpm1.position_change;

            // set the product index of the product in this entry
            cache.product_indexes[variance_cache_index] = tpm1.product_index;
            // set the position that we used to calculate sigma_p
            cache.positions[variance_cache_index] = tpm1.position_change;
            // set the sigma_p (p == q in this case) value
            cache.sigma_position[variance_cache_index] = variance_term;
        }

        for variance_cache_index in q.len()..cache.product_indexes.len() {
            cache.product_indexes[variance_cache_index] = usize::MAX;
            cache.positions[variance_cache_index] = ZERO_FAST_INT;
            cache.sigma_position[variance_cache_index] = ZERO_FAST_INT;
        }

        // now calculate open order variance
        cache.open_order_variance = Self::calculate_open_order_variance(
            &market_product_group,
            &trader_risk_group,
            &cache,
            &q,
            &covariance_matrix,
        )?;

        Ok(())
    }

    pub fn calculate_risk_from_cache(
        market_product_group: &MarketProductGroup<S>,
        mark_prices: &MarkPricesArray<S>,
        trader_risk_group: &TraderRiskGroup<S>,
        covariance_matrix: &CovarianceMatrix,
        cache: &mut VarianceCache,
        // abs_position_value: &mut [FastInt; MAX_TRADER_POSITIONS],
        abs_position_value: &mut Vec<FastInt>,
        total_abs_position_value: &mut FastInt,
        current_slot: Slot,
    ) -> Result<()> {
        // TODO: can possibly short-circuit a lot of the position calculation if position_change == 0,
        // TODO: because we are now storing things with open orders as well
        // TODO: same goes for open orders processing

        // this is meant to represent the vector of positions that have changed (e)
        // (product_index, change in position, bid qty, ask qty, array index of the product in our VarianceCache)
        let mut e: Vec<TraderPositionMetadata> = Vec::with_capacity(MAX_TRADER_POSITIONS);

        // use this as a record of the variance cache indexes that we need/don't need to check for removal
        let mut matched_variance_cache_indexes = [false; 2 * MAX_TRADER_POSITIONS];

        // reset the position value, which we will calculate in the initial loop
        cache.derivative_position_value = ZERO_FAST_INT;
        cache.total_liquidity_buffer = FastInt::from(-1);

        let mut liquidity_buffer_usd: Option<FastInt> = Some(FastInt::from(0));

        // put together the 'e' vector of new positions
        for (trader_position_index, trader_position) in
            trader_risk_group.trader_positions.iter().enumerate()
        {
            if !trader_position.is_initialized() {
                continue;
            }

            let product_index: usize = trader_position.product_index;
            let base_decimals =
                market_product_group.active_products.array[product_index].base_decimals; // TODO: figure this out
            let position: FastInt =
                (trader_position.position + trader_position.pending_position).into();
            let (open_bid_qty, open_ask_qty) = {
                let open_bid_qty_book: FastInt = Fractional::new(
                    trader_risk_group.open_orders.products[product_index].bid_qty_in_book,
                    base_decimals,
                )
                .into();
                let open_ask_qty_book: FastInt = Fractional::new(
                    trader_risk_group.open_orders.products[product_index].ask_qty_in_book,
                    base_decimals,
                )
                .into();
                let open_bid_qty_lock: FastInt = trader_risk_group.locked_collateral
                    [trader_position_index]
                    .bid_qty
                    .into();
                let open_ask_qty_lock: FastInt = trader_risk_group.locked_collateral
                    [trader_position_index]
                    .ask_qty
                    .into();
                (
                    open_bid_qty_book + open_bid_qty_lock,
                    open_ask_qty_book + open_ask_qty_lock,
                )
            };

            // now attempt to find the previous position
            let variance_cache_index = find_product_index(&cache, product_index);

            let prev_position = match variance_cache_index {
                Some(i) => {
                    matched_variance_cache_indexes[i] = true;
                    cache.positions[i]
                }
                None => ZERO_FAST_INT,
            };

            let position_change = position - prev_position;

            // we care about processing this position if there was a position change or we have an open order
            // THE ABOVE COMMENT IS NO LONGER TRUE^ because even if no bid, no ask, and no position, this product could be a leg of a combo that DOES have an open bid or open ask...
            // ... therefore, the following if statement is commented out.
            // if position_change != ZERO_FAST_INT || open_bid_qty.m != 0 || open_ask_qty.m != 0 {
            e.push(TraderPositionMetadata {
                product_index,
                position_change,
                open_bid_qty,
                open_ask_qty,
                variance_cache_index,
            });
            // }

            // skip this processing for products where we have 0 position but have open orders
            if position != ZERO_FAST_INT {
                // calculate price of this derivative
                let product = &market_product_group.active_products[product_index];
                let price =
                    mark_prices.calculate_price(market_product_group, product, current_slot)?;

                let this_position_value = price.mul_clamp_to_tick(position);
                let this_position_value_abs = this_position_value.abs();

                // accumulate total derivative position value
                cache.derivative_position_value += this_position_value;

                // store the abs(position_value) at the product_index, and accumulate the total
                abs_position_value[product_index] = this_position_value_abs;
                *total_abs_position_value += this_position_value_abs;

                if !product.is_combo() {
                    match liquidity_buffer_usd {
                        Some(ref mut buffer) => {
                            let mp_product_index =
                                mark_prices.get_product_index(&product.product_id)?; // TODO: figure this out
                            let mark_price = price.to_frac().unwrap();

                            let (is_long, qualifying_price): (bool, Option<Fractional>) =
                                if position > ZERO_FAST_INT {
                                    (
                                        true,
                                        mark_prices.array[mp_product_index].qualifying_bid_price,
                                    )
                                } else {
                                    (
                                        false,
                                        mark_prices.array[mp_product_index].qualifying_ask_price,
                                    )
                                };

                            let px = match qualifying_price {
                                Some(q_px) => Some(q_px),
                                None => {
                                    let outright = product.try_to_outright().unwrap();
                                    if outright.mark_price_qualifying_cum_value == ZERO_FRAC {
                                        match mark_prices.calculate_outright_book_price(outright) {
                                            Ok(px) => Some(px.to_frac().unwrap()),
                                            _ => None,
                                        }
                                    } else {
                                        None
                                    }
                                }
                            };

                            match px {
                                Some(book_price) => {
                                    if (is_long && mark_price > book_price)
                                        || (!is_long && mark_price < book_price)
                                    {
                                        *buffer += FastInt::from(
                                            (position.to_frac().unwrap()
                                                * (mark_price - book_price))
                                                .abs(),
                                        );
                                    }
                                }
                                None => {
                                    liquidity_buffer_usd = None;
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        }

        if !liquidity_buffer_usd.is_none() {
            cache.total_liquidity_buffer = liquidity_buffer_usd.unwrap();
        }

        let mut removed_array_indexes: Vec<usize> = vec![];

        // go through our cache, try to determine if there were any products that we previously had a position
        // in, but that no longer exists in the TraderPositions
        for (array_index, product_index) in cache.product_indexes.iter().enumerate() {
            // skip non-active elements
            if *product_index == usize::MAX {
                continue;
            }
            // skip elements that we know we found in the trader position loop
            if matched_variance_cache_indexes[array_index] {
                continue;
            }

            // try to find this product in the current trader positions
            // TODO: I think we can skip this, because this should ALWAYS be false if it wasn't matched above?
            let mut found = false;
            for trader_position in trader_risk_group.trader_positions.iter() {
                if !trader_position.is_initialized() {
                    continue;
                }
                if trader_position.product_index == *product_index {
                    found = true;
                    break;
                }
            }

            if found {
                // we shouldn't ever get here:
                //    this case indicates that there was an entry in the existing variance cache that wasn't found when
                //    iterating through TraderPositions, but then wnen iterating through existing variance cache positions,
                //    it wasn't marked as matched and then we found a corresponding TraderPosition
                //     msg!("ERROR: Unexpected result, unmatched cache/position, product_index={}, position={}, array_index={}",
                //     *product_index,
                //     cache.positions[array_index],
                //     array_index,
                // );
                return Err(RiskError::UnexpectedResult.into());
            }

            // msg!("Found a missing previous position, product_index={}, position_change={}, array_index={}",
            //     *product_index,
            //     -cache.positions[array_index],
            //     array_index,
            // );

            // get the open orders data for this product
            let open_orders_metdata = &trader_risk_group.open_orders.products[*product_index];
            let base_decimals =
                market_product_group.active_products.array[*product_index].base_decimals; // TODO: figure this out

            e.push(TraderPositionMetadata {
                product_index: *product_index,
                // position went to 0, so our change is -1 * previous existing position
                position_change: -cache.positions[array_index],
                // TODO: I'm fairly confident that open bid/ask should be ZERO_FRAC if the TraderPosition has been removed
                open_bid_qty: Fractional::new(open_orders_metdata.bid_qty_in_book, base_decimals)
                    .into(),
                open_ask_qty: Fractional::new(open_orders_metdata.ask_qty_in_book, base_decimals)
                    .into(),
                variance_cache_index: Some(array_index),
            });

            // we'll set this variance cache slot to unused after all of our processing is done
            removed_array_indexes.push(array_index);
        }

        // extend sigma_p for new products
        for tpm1 in e.iter_mut() {
            // the optional here indicates that it doesn't have a current slot in the variance cache
            if tpm1.variance_cache_index.is_none() {
                let array_index = find_open_array_index(&cache)?;

                // calculate vs. previous position
                let mut total_p_covariance = ZERO_FAST_INT;
                for (e2_array_index, e2_product_index) in cache.product_indexes.iter().enumerate() {
                    if *e2_product_index < usize::MAX {
                        let prev_position = cache.positions[e2_array_index];
                        if prev_position != ZERO_FAST_INT {
                            total_p_covariance +=
                                covariance_matrix.get_covariance_from_product_indexes(
                                    tpm1.product_index,
                                    *e2_product_index,
                                )? * prev_position;
                        }
                    }
                }
                cache.product_indexes[array_index] = tpm1.product_index;
                cache.sigma_position[array_index] = total_p_covariance;
                tpm1.variance_cache_index = Some(array_index);
            }
        }

        // compute sigma_e
        let mut sigma_e = [ZERO_FAST_INT; 2 * MAX_TRADER_POSITIONS];
        for (e1_array_index, e1_product_index) in cache.product_indexes.iter().enumerate() {
            if *e1_product_index == usize::MAX {
                continue;
            }

            let mut total_covariance = ZERO_FAST_INT;
            for tpm2 in e.iter() {
                // only necessary when the position change != 0
                if tpm2.position_change != ZERO_FAST_INT {
                    let covariance = covariance_matrix.get_covariance_from_product_indexes(
                        *e1_product_index,
                        tpm2.product_index,
                    )?;
                    total_covariance += covariance * tpm2.position_change;
                }
            }
            sigma_e[e1_array_index] = total_covariance;
        }

        // calculate both (e.T * sigma * e) and (e.T * sigma * p) in the same loop
        let mut e_sigma_p = ZERO_FAST_INT;
        let mut e_sigma_e = ZERO_FAST_INT;
        for tpm in e.iter() {
            e_sigma_p += tpm
                .position_change
                .checked_mul(cache.sigma_position[tpm.variance_cache_index.unwrap()])?;
            e_sigma_e += tpm
                .position_change
                .checked_mul(sigma_e[tpm.variance_cache_index.unwrap()])?;
        }

        // q.T * sigma * q = (p.T * sigma * p) + (e.T * sigma * e) + (2 * e.T * sigma * p)
        cache.total_variance_traded =
            cache.total_variance_traded + e_sigma_e + TWO_FAST_INT * e_sigma_p;

        // now update the cache such that sigma * q = (sigma * p) + (sigma * e)
        for (p, e) in cache.sigma_position.iter_mut().zip(sigma_e.iter()) {
            *p += *e;
        }

        // update our position vector, q, which will be p for the next iteration
        for tpm in e.iter() {
            cache.positions[tpm.variance_cache_index.unwrap()] += tpm.position_change;
        }

        // and finally, set all of the 'inactive products' to inactive
        for array_index in removed_array_indexes.iter() {
            cache.product_indexes[*array_index] = usize::MAX;
            cache.positions[*array_index] = ZERO_FAST_INT;
            cache.sigma_position[*array_index] = ZERO_FAST_INT;
        }

        // now calculate open order variance
        cache.open_order_variance = Self::calculate_open_order_variance(
            &market_product_group,
            &trader_risk_group,
            &cache,
            &e,
            &covariance_matrix,
        )?;

        Ok(())
    }

    pub fn calculate_liquidation_status(
        risk_profile: &RiskProfile,
        trader_risk_group: &TraderRiskGroup<S>,
        health_status: HealthStatus,
    ) -> Result<LiquidationStatus, RiskError> {
        // i would make this a constant, but for whatever reason the compiler
        // crashes if it is defined as:
        // pub const ZERO_SOCIAL_LOSS: SocialLoss = SocialLoss {
        //    product_index: MAX_OUTRIGHTS,
        //    amount: ZERO_FRAC,
        //}
        let zero_social_loss: SocialLossInfo = SocialLossInfo {
            product_idx: MAX_OUTRIGHTS,
            amount: ZERO_FRAC,
        };

        match health_status {
            HealthStatus::Liquidatable => {
                let liquidation_price =
                    risk_profile.portfolio_value - risk_profile.portfolio_std_dev;
                let total_social_loss = (-liquidation_price).max(ZERO_FAST_INT);
                // liquidatee's cash and positions are transferred in their entirety to liquidator,
                // who must have enough collateral to cover these positions for this tx to succeed.
                // The $100 TSL comes from $100 being paid to the liquidator (initially by the liquidatee,
                // but now that they have no positions or cash, this is paid via TSL).

                let mut liquidation_status = LiquidationStatus {
                    health_status: HealthStatus::Liquidatable,
                    action_result: ActionStatus::Approved,
                    total_social_loss: total_social_loss.to_frac().unwrap(),
                    liquidation_price: liquidation_price.to_frac().unwrap(),
                    social_losses: vec![zero_social_loss; MAX_TRADER_POSITIONS],
                };

                for (tpi, trader_position) in trader_risk_group.trader_positions.iter().enumerate()
                {
                    if !trader_position.is_initialized() {
                        continue;
                    }

                    let product_social_loss = total_social_loss
                        * risk_profile.abs_position_value[trader_position.product_index]
                        / risk_profile.total_abs_position_value;

                    liquidation_status.social_losses[tpi] = SocialLossInfo {
                        product_idx: trader_position.product_index,
                        amount: product_social_loss.to_frac().unwrap(),
                    }
                }

                Ok(liquidation_status)
            }
            HealthStatus::Unhealthy => Ok(LiquidationStatus {
                health_status: HealthStatus::Unhealthy,
                action_result: ActionStatus::NotApproved,
                total_social_loss: ZERO_FRAC,
                liquidation_price: ZERO_FRAC,
                social_losses: vec![zero_social_loss; MAX_TRADER_POSITIONS],
            }),
            HealthStatus::Healthy => Ok(LiquidationStatus {
                health_status: HealthStatus::Healthy,
                action_result: ActionStatus::NotApproved,
                total_social_loss: ZERO_FRAC,
                liquidation_price: ZERO_FRAC,
                social_losses: vec![zero_social_loss; MAX_TRADER_POSITIONS],
            }),
            _ => Err(RiskError::UnexpectedResult.into()),
        }
    }

    // calculates the total additional variance from open orders
    // for each position in position_metadata:
    //   additional variance will be the max ADDITIONAL variance (q + o).T * sigma * (q + o)
    //   where q:current position and o:{open_bid_qty, open_ask_qty}
    //
    //   we can write additional variance = (q + o).T * sigma * (q + o) - q * sigma * q
    //     = q.T * sigma * q + 2 * o.T * sigma * q + o.T * sigma * o - q * sigma * q
    //     = 2 * o.T * sigma * q + o.T * sigma * o
    //     (and we already know sigma * q)
    fn calculate_open_order_variance(
        market_product_group: &MarketProductGroup<S>,
        trader_risk_group: &TraderRiskGroup<S>,
        cache: &VarianceCache,
        position_metadata: &Vec<TraderPositionMetadata>,
        covariance_matrix: &CovarianceMatrix,
    ) -> Result<FastInt, RiskError> {
        let mut open_order_variance = ZERO_FAST_INT;

        // additional variance = 2 * o.T * sigma * q + o.T * sigma * o
        // where o is the open order vector (indiviual instruments, and individual sides Bid/Ask)
        for tpm in position_metadata.iter() {
            if tpm.open_bid_qty == ZERO_FAST_INT && tpm.open_ask_qty == ZERO_FAST_INT {
                continue;
            }

            let variance = covariance_matrix
                .get_covariance_from_product_indexes(tpm.product_index, tpm.product_index)?;

            let open_bid_addtl_var = match tpm.open_bid_qty == ZERO_FAST_INT {
                true => ZERO_FAST_INT,
                // sigma_p in our variance cache has now been updated to reflect (q), and is therefore sigma_q
                false => {
                    tpm.open_bid_qty * variance * tpm.open_bid_qty
                        + (TWO_FAST_INT * tpm.open_bid_qty)
                            .mul_zero_okay(cache.sigma_position[tpm.variance_cache_index.unwrap()])
                }
            };
            let open_ask_addtl_var = match tpm.open_ask_qty == ZERO_FAST_INT {
                true => ZERO_FAST_INT,
                // we subtract here because the ask quantity is represented as a + number, but in actually would result in a - position
                // instead of multiplying by -1, we just subtract here (it doesn't matter in the variance calc b/c it is squared, and
                // therefore open_ask^2 will always be a positive number)
                false => {
                    tpm.open_ask_qty * variance * tpm.open_ask_qty
                        - (TWO_FAST_INT * tpm.open_ask_qty)
                            .mul_zero_okay(cache.sigma_position[tpm.variance_cache_index.unwrap()])
                }
            };

            open_order_variance += open_bid_addtl_var
                .max(open_ask_addtl_var)
                .max(ZERO_FAST_INT);
        }

        let mut combo_deltas: Vec<ComboDelta> = vec![];
        for (combo_index, combo) in market_product_group.active_combo_products() {
            let open_orders_metdata = &trader_risk_group.open_orders.products[combo_index];
            let base_decimals =
                market_product_group.active_products.array[combo_index].base_decimals;
            if open_orders_metdata.bid_qty_in_book != 0 {
                combo_deltas.push(ComboDelta {
                    combo: combo,
                    position_change: Fractional::new(
                        open_orders_metdata.bid_qty_in_book,
                        base_decimals,
                    )
                    .into(),
                });
            }
            if open_orders_metdata.ask_qty_in_book != 0 {
                combo_deltas.push(ComboDelta {
                    combo: combo,
                    position_change: -FastInt::from(Fractional::new(
                        open_orders_metdata.ask_qty_in_book,
                        base_decimals,
                    )),
                });
            }
        }

        let mut open_combo_variance = ZERO_FAST_INT;
        for combination_size in 0..combo_deltas.len() + 1 {
            for combination in Self::combinations(&combo_deltas, combination_size).unwrap() {
                let this_variance = Self::calculate_open_combo_orders_variance(
                    &cache,
                    &combination,
                    &covariance_matrix,
                )?;
                if this_variance > open_combo_variance {
                    open_combo_variance = this_variance;
                }
            }
        }
        open_order_variance += open_combo_variance;

        Ok(open_order_variance)
    }

    fn combinations<T>(
        collection: &Vec<T>,
        combination_size: usize,
    ) -> Result<Combinations<T>, RiskError> {
        if combination_size > collection.len() {
            return Err(RiskError::ComboSizeGreaterThanCollectionLen.into());
        }
        Ok(Combinations::<T> {
            collection,
            indices: (0..combination_size).collect(),
            is_done: false,
        })
    }

    fn calculate_open_combo_orders_variance<'a, P: AsRef<ComboDelta<'a>>>(
        cache: &VarianceCache,
        combo_deltas: &Vec<P>,
        covariance_matrix: &CovarianceMatrix,
    ) -> Result<FastInt, RiskError> {
        let mut product_deltas = [ZERO_FAST_INT; MAX_TRADER_POSITIONS];
        for delta in combo_deltas.iter() {
            let delta = delta.as_ref();
            if delta.position_change == ZERO_FAST_INT {
                continue;
            }
            for leg in delta.combo.legs().iter() {
                product_deltas[leg.product_index] += leg.ratio * delta.position_change;
            }
        }
        let mut additional_variance = ZERO_FAST_INT;
        for (i, p1) in cache.positions.iter().enumerate() {
            if *p1 == ZERO_FAST_INT {
                continue;
            }
            for (j, d1) in product_deltas.iter().enumerate() {
                if *d1 == ZERO_FAST_INT {
                    continue;
                }
                let covariance = covariance_matrix.get_covariance_from_product_indexes(
                    cache.product_indexes[i],
                    j, // j is already a product index
                )?;
                additional_variance += *d1 * TWO_FAST_INT * covariance * *p1;
            }
        }
        for (i, d1) in product_deltas.iter().enumerate() {
            if *d1 == ZERO_FAST_INT {
                continue;
            }
            for (j, d2) in product_deltas.iter().enumerate() {
                if *d2 == ZERO_FAST_INT {
                    continue;
                }
                let covariance = covariance_matrix.get_covariance_from_product_indexes(i, j)?;
                additional_variance += *d1 * covariance * *d2;
            }
        }
        Ok(ZERO_FAST_INT.max(additional_variance))
    }
}

// finds an open array index in the variance cache's list of product indexes
// returns:
//   Result::Ok(array_index) upon finding an open slot
//   Result::Err if all slots are open (shouldn't occur since we're using 2x available slots as positions)
#[inline(always)]
pub fn find_open_array_index(cache: &VarianceCache) -> Result<usize, RiskError> {
    for (array_index, cache_product_index) in cache.product_indexes.iter().enumerate() {
        if *cache_product_index == usize::MAX {
            return Ok(array_index);
        }
    }
    return Err(RiskError::UnexpectedResult.into());
}

// find a specified product_indx in the variance cache by iterating through the list of product_indexes
// in the variance cache and returning the array index of the matched product index.
// returns:
//    Option::Some(array_index) upon finding a match
//    Option::None in the case a match is not found
#[inline(always)]
pub fn find_product_index(cache: &VarianceCache, product_index: usize) -> Option<usize> {
    for (array_index, cache_product_index) in cache.product_indexes.iter().enumerate() {
        if product_index == *cache_product_index {
            return Option::Some(array_index);
        }
    }
    return Option::None;
}

struct Combinations<'a, T> {
    collection: &'a Vec<T>,
    indices: Vec<usize>,
    is_done: bool,
}

impl<'a, T> Iterator for Combinations<'a, T> {
    type Item = Vec<&'a T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }
        let mut combination = vec![];
        for i in &self.indices {
            combination.push(&self.collection[*i]);
        }
        if self.indices.len() == 0 {
            self.is_done = true;
            return Some(combination);
        }
        let mut ii = self.indices.len() - 1;
        loop {
            let max_index = self.collection.len() - (self.indices.len() - ii);
            self.indices[ii] += 1;
            if self.indices[ii] > max_index {
                if ii == 0 {
                    self.is_done = true;
                    break;
                }
                self.indices[ii] = self.indices[ii - 1] + 2;
                if self.indices[ii] > max_index {
                    self.indices[ii] = max_index;
                }
                ii -= 1;
            } else {
                break;
            }
        }
        Some(combination)
    }
}

struct ComboDelta<'a> {
    combo: &'a ComboProduct,
    // open bid or open ask qty
    position_change: FastInt,
}

impl<'a> AsRef<ComboDelta<'a>> for ComboDelta<'a> {
    fn as_ref(&self) -> &ComboDelta<'a> {
        self
    }
}
