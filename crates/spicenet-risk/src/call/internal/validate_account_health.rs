use anyhow::{Error, Result};
use sov_modules_api::{Context, EventEmitter, Spec, TxState};

use spicenet_shared::dex::{MarketProductGroup, TraderRiskGroup};
use spicenet_shared::risk::RiskError;
use spicenet_shared::Side;
use spicenet_shared::{FastInt, Fractional, ZERO_FAST_INT};

use crate::event::Event;
use crate::state::RiskProfile;
use crate::RiskModule;
use spicenet_shared::risk::health_status::HealthStatus;
use spicenet_shared::risk::{
    ActionStatus, HealthOutput, HealthTracker, RiskEngineOpCodes, RiskInfo,
};

/// ATTENTION: Update MIN_ESCAPABALE_PRICE when changing PRICE_BAND_PROPORTION.
/// We could probably do this at compile time with constexprs somehow
/// but I don't want to take the time to figure that out right now.
pub const PRICE_BAND_PROPORTION: Fractional = Fractional { m: 15, exp: 2 };
pub const IS_PRICE_BANDS_ENABLED: bool = true;

/// ATTENTION: change this value when changing PRICE_BAND_PROPORTIO     N
/// This value is calculated to be the minimum positive price such that
/// a trader can place an order with a _different_ price without violating
/// price bands. See that if the mark price is positive and less than this number,
/// then any price != mark price will be more than 15% away and thus a price band
/// violation.
pub const MIN_ESCAPABLE_PRICE: FastInt = FastInt { value: 000_0006 };

impl<S: Spec> RiskModule<S> {
    pub fn validate_account_health(
        &self,
        mpg: &mut MarketProductGroup<S>,
        trg: TraderRiskGroup<S>,
        params: RiskInfo,
        _context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<()> {
        // let mpg = self
        //     .dex
        //     .get_mpg(state, &mpg_id)
        //     .or_else(|_| Err(DexError::MarketProductGroupDoesNotExist.into()))?;

        // let trg = self
        //     .dex
        //     .get_trg(state, &trg_id)
        //     .or_else(|_| Err(DexError::TraderRiskGroupDoesNotExist.into()))?;

        let mut covariance_matrix = self
            .covariance_matrix
            .get(&mpg.id, state)
            .unwrap()
            .ok_or::<Error>(RiskError::CovarianceMatrixNotInitialized.into())?;
        let mark_prices = self
            .mark_prices
            .get(&mpg.id, state)
            .unwrap()
            .ok_or::<Error>(RiskError::MarkPricesNotInitialized.into())?;

        let metadata = &covariance_matrix.covariance_metadata;
        let mut variance_cache = self
            .variance_caches
            .get(&trg.id, state)
            .unwrap()
            .ok_or::<Error>(RiskError::VarianceCacheNotInitialized.into())?;

        if IS_PRICE_BANDS_ENABLED && params.op_type == RiskEngineOpCodes::NewOrder {
            if params.num_orders > 0 {
                return Err(RiskError::InvalidRiskCheckParameters.into());
            }

            for order in params.orders.iter().take(params.num_orders as usize) {
                if order.is_combo {
                    continue;
                }
                let product = &mpg.active_products.array[order.idx];
                let product_mark_price = mark_prices.calculate_price(
                    &mpg,
                    product,
                    self.time_module.get_slot(state).unwrap().slot,
                )?;

                if product_mark_price > MIN_ESCAPABLE_PRICE {
                    let lower_band = Fractional::from(1)
                        .checked_sub(PRICE_BAND_PROPORTION)?
                        .checked_mul(product_mark_price.to_frac()?)?;
                    let upper_band = Fractional::from(1)
                        .checked_add(PRICE_BAND_PROPORTION)?
                        .checked_mul(product_mark_price.to_frac()?)?;
                    if (order.side == Side::Ask && order.order_price < lower_band)
                        || (order.side == Side::Bid && order.order_price > upper_band)
                    {
                        //     msg!(
                        //     "price bands: ({}, {}); limit price: {}; mark price: {}",
                        //     lower_band,
                        //     upper_band,
                        //     order.order_price,
                        //     product_mark_price.to_frac()?,
                        // );
                        return Err(RiskError::PriceBandViolation.into());
                    }
                } else {
                    // msg!("ignoring price bands because mark price < minimum escapable price");
                }
            }
        }

        for variance_cache_index in 0..variance_cache.product_indexes.len() {
            variance_cache.product_indexes[variance_cache_index] = usize::MAX;
            variance_cache.positions[variance_cache_index] = ZERO_FAST_INT;
            variance_cache.sigma_position[variance_cache_index] = ZERO_FAST_INT;
        }

        covariance_matrix.mappings =
            Self::map_all_indexes(&metadata, &mpg, &trg, &variance_cache).unwrap();

        let old_risk_profile: RiskProfile = (&variance_cache, &trg).into();
        let risk_profile = Self::calculate_risk_profile_cached(
            &mpg,
            &mark_prices,
            &trg,
            &covariance_matrix,
            &mut variance_cache,
            self.time_module.get_slot(state).unwrap().slot,
        )?;

        let mut block_withdrawal = false;
        if params.op_type == RiskEngineOpCodes::CheckWithdrawHealth
            && (variance_cache.total_liquidity_buffer < ZERO_FAST_INT
                || variance_cache.total_liquidity_buffer > risk_profile.portfolio_value)
        {
            // msg!("blocking withdrawal due to liquidity buffer");
            // msg!(
            //     "total_liquidity_buffer: {}. portfolio_value: {}",
            //     &variance_cache.total_liquidity_buffer.to_frac().unwrap(),
            //     &risk_profile.portfolio_value.to_frac().unwrap()
            // );
            block_withdrawal = true;
        }

        let health_output = match risk_profile.get_health_status() {
            HealthStatus::Liquidatable => HealthOutput::Healthy {
                health_status: HealthTracker {
                    health_status: HealthStatus::Liquidatable,
                    action_status: ActionStatus::NotApproved,
                },
            },
            HealthStatus::Unhealthy => HealthOutput::Healthy {
                health_status: HealthTracker {
                    health_status: HealthStatus::Unhealthy,
                    action_status: if !block_withdrawal
                        && risk_profile.portfolio_open_order_std_dev
                            <= old_risk_profile.portfolio_open_order_std_dev
                    {
                        ActionStatus::Approved
                    } else {
                        ActionStatus::NotApproved
                    },
                },
            },
            HealthStatus::Healthy => HealthOutput::Healthy {
                health_status: HealthTracker {
                    health_status: HealthStatus::Healthy,
                    action_status: if !block_withdrawal {
                        ActionStatus::Approved
                    } else {
                        ActionStatus::NotApproved
                    },
                },
            },
            _ => return Err(RiskError::UnexpectedResult.into()),
        };

        mpg.risk_output_register.health_output = health_output.clone();

        self.emit_event(
            state,
            Event::AccountHealthValidation {
                mpg_id: mpg.id,
                trg_id: trg.id,
                risk_info_params: params,
                block_withdrawal,
                health_output,
            },
        );

        Ok(())
    }
}
