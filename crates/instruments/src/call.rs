use sov_modules_api::{Address, CallResponse, Context, EventEmitter, Spec, TxState};

use spicenet_shared::{Fractional, MPGId, UtilError, ZERO_FRAC};

use spicenet_shared::dex::constants::{NO_ASK_PRICE, NO_BID_PRICE};
use spicenet_shared::dex::{DexError, Product};
use std::error;

use crate::derivative_metadata::DerivativeMetadata;
use crate::{
    error::DerivativeError,
    get_derivative_id,
    state::enums::{ExpirationStatus, InstrumentType, OracleType},
    Event, Instruments,
};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct InitializeDerivativeParams<S: Spec> {
    pub derivative_name: String,
    pub instrument_type: InstrumentType,
    pub strike: Fractional,
    pub full_funding_period: u64,
    pub minimum_funding_period: u64,
    pub initialization_time: u64,
    pub close_authority: Address<S>,
    pub oracle_type: OracleType,
    pub price_oracle: u32,
    pub market_product_group: MPGId,
    pub expiration_status: ExpirationStatus,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct CloseDerivativeParams {
    pub derivative_name: String,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct SettleDerivativeParams {
    pub derivative_name: String,
    pub index_settle_recurring_derivative: bool,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub enum CallMessage<S: Spec> {
    InitDerivative {
        params: InitializeDerivativeParams<S>,
    },
    CloseDerivative {
        params: CloseDerivativeParams,
    },
    SettleDerivative {
        params: SettleDerivativeParams,
    },
}

impl<S: Spec> Instruments<S> {
    pub(crate) fn close_derivative(
        &self,
        params: CloseDerivativeParams,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse, DexError> {
        let derivative_id = get_derivative_id::<S>(params.derivative_name.as_str());
        let derivative = &self.derivative_metadata.get(&derivative_id, state).unwrap();
        if derivative.is_none() {
            Err(DexError::DerivativeNotFound)
        } else {
            &self.derivative_metadata.remove(&derivative_id, state);
            Ok(CallResponse::default())
        }
    }

    pub(crate) fn initialize_derivative(
        &self,
        params: InitializeDerivativeParams<S>,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse, DexError> {
        // self.validate_initialize_derivative(&params, context, state)?;

        let derivative_id = get_derivative_id::<S>(&params.derivative_name);
        let mut derivative_metadata = DerivativeMetadata::new(
            params.derivative_name.clone(),
            params.instrument_type,
            params.strike,
            params.initialization_time,
            params.full_funding_period,
            params.minimum_funding_period,
            params.close_authority,
            params.market_product_group,
            params.price_oracle,
            params.oracle_type,
        );

        // Set the expiration status from the params
        derivative_metadata.expiration_status = params.expiration_status;
        derivative_metadata.last_funding_time = params.initialization_time;

        self.derivative_metadata
            .set(&derivative_id, &derivative_metadata, state);

        self.emit_event(state, Event::DerivativeInitalized { id: derivative_id });

        Ok(CallResponse::default())
    }

    pub(crate) fn settle_derivative(
        &self,
        params: SettleDerivativeParams,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse, DexError> {
        self.validate_settle_derivative(&params, context, state);

        let derivative_id = get_derivative_id(params.derivative_name.as_str());
        let mut derivative_metadata = &self
            .derivative_metadata
            .get(&derivative_id, state)
            .unwrap()
            .unwrap();

        let funding_amount = &self.get_funding_amount(
            &mut derivative_metadata,
            params.index_settle_recurring_derivative,
            context,
            state,
        );

        self.dex
            .update_product_funding_in_dex(&derivative_metadata, funding_amount, state)?;

        self.derivative_metadata
            .set(&derivative_id, &derivative_metadata, state);

        self.emit_event(state, Event::DerivativeSettled { id: derivative_id });

        Ok(CallResponse::default())
    }

    pub(crate) fn validate_settle_derivative(
        &self,
        params: &SettleDerivativeParams,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<(), Box<dyn error::Error>> {
        let derivative_id = get_derivative_id(params.derivative_name.as_str());
        let derivative_metadata = self
            .derivative_metadata
            .get(&derivative_id, state)?
            .ok_or(DexError::DerivativeNotFound)?;

        if !derivative_metadata.is_initialized() {
            return Err(DerivativeError::UninitializedAccount.into());
        }

        if derivative_metadata.is_expired_or_expiring() {
            return Err(DerivativeError::ContractIsExpired.into());
        }

        if params.index_settle_recurring_derivative {
            let market_product_group = self
                .dex
                .market_product_groups
                .get(&derivative_metadata.market_product_group, state)?;
            // TODO: Check wallets and signature related stuff later
            // if !context.is_signed_by(&market_product_group.unwrap().mpg_authority) {
            //     return Err(DerivativeError::Unauthorized.into());
            // }
        }

        Ok(())
    }

    fn get_funding_amount(
        &self,
        derivative_metadata: &mut DerivativeMetadata<S>,
        index_settle_recurring_derivative: bool,
        context: &Context<S>,
        state: &mut impl TxState<S>,
    ) -> Result<Fractional, Box<dyn error::Error>> {
        let current_time = self.time.get_time(state).unwrap();

        if current_time.unix_timestamp <= derivative_metadata.initialization_time {
            return Err(DerivativeError::InvalidSettlementTime.into());
        }

        let elapsed = current_time.unix_timestamp - derivative_metadata.last_funding_time;
        if elapsed >= derivative_metadata.minimum_funding_period
            || index_settle_recurring_derivative
        {
            derivative_metadata.last_funding_time = current_time.unix_timestamp.into();
        } else {
            return Err(DerivativeError::InvalidSettlementTime.into());
        }

        let index_price = if derivative_metadata.instrument_type.is_recurring()? {
            self.oracle.get_ema(0, state)?.ema
        } else {
            self.oracle.get_price(0, state)?.price
        };

        let payoff = self.get_payoff(derivative_metadata, index_price)?;

        if derivative_metadata.instrument_type.is_recurring()? && index_settle_recurring_derivative
        {
            Ok(payoff)
        } else if derivative_metadata.instrument_type.is_recurring()? {
            let mark_price = self.get_mark_price(derivative_metadata.active_products[0], state)?;
            let offset = payoff - mark_price;
            let num = Fractional::from(elapsed.into());
            let denom = Fractional::from(derivative_metadata.full_funding_period.into());
            let pct = (num / (denom)).min(Fractional::new(1, 0));
            Ok((offset * pct).round_sf(2))
        } else {
            derivative_metadata.expiration_status = ExpirationStatus::Expiring;
            Ok(payoff)
        }
    }

    pub fn get_payoff(
        &self,
        derivative_metadata: &DerivativeMetadata<S>,
        index_price: Fractional,
    ) -> Result<Fractional, UtilError> {
        let raw_payoff = match derivative_metadata.instrument_type {
            InstrumentType::RecurringCall | InstrumentType::ExpiringCall => {
                index_price - derivative_metadata.strike
            }
            InstrumentType::RecurringPut | InstrumentType::ExpiringPut => {
                derivative_metadata.strike - index_price
            }
            _ => {
                return Err(UtilError::AccountUninitialized);
            }
        };
        Ok(if raw_payoff.is_negative() {
            ZERO_FRAC
        } else {
            raw_payoff
        })
    }

    fn get_mark_price(
        &self,
        market_product: Product,
        state: &mut impl TxState<S>,
    ) -> Result<Fractional, DexError> {
        let best_bid = market_product.get_best_bid();
        let best_ask = market_product.get_best_ask();
        if best_ask == NO_ASK_PRICE || best_bid == NO_BID_PRICE {
            return Err(DexError::InvalidAccountData);
        }
        Ok((best_bid + best_ask) / (Fractional::new(2, 0)))
    }
}
