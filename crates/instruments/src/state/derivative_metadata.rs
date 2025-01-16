use sov_modules_api::{Address, Spec};

use spicenet_shared::{Fractional, MPGId};

use crate::state::enums::{ExpirationStatus, InstrumentType, OracleType};

pub type UnixTimestamp = u64;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct DerivativeMetadata<S: Spec> {
    pub derivative_name: String,
    pub expiration_status: ExpirationStatus,
    pub oracle_type: OracleType,
    pub instrument_type: InstrumentType,
    pub bump: u64,
    pub strike: Fractional,
    pub initialization_time: UnixTimestamp,
    pub full_funding_period: UnixTimestamp,
    pub minimum_funding_period: UnixTimestamp,
    pub price_oracle: u64,
    pub market_product_group: MPGId,
    pub close_authority: Address<S>,
    pub clock: u16,
    pub last_funding_time: UnixTimestamp,
}

impl<S: Spec> DerivativeMetadata<S> {
    pub fn new(
        derivative_name: String,
        instrument_type: InstrumentType,
        strike: Fractional,
        initialization_time: UnixTimestamp,
        full_funding_period: UnixTimestamp,
        minimum_funding_period: UnixTimestamp,
        close_authority: Address<S>,
        market_product_group: MPGId,
        price_oracle: u32,
        oracle_type: OracleType,
    ) -> Self {
        Self {
            derivative_name: derivative_name,
            expiration_status: ExpirationStatus::default(),
            oracle_type,
            instrument_type,
            bump: 0,
            strike,
            initialization_time,
            full_funding_period,
            minimum_funding_period,
            price_oracle: price_oracle.into(),
            market_product_group,
            close_authority,
            clock: 0,
            last_funding_time: initialization_time,
        }
    }
}

impl<S: Spec> DerivativeMetadata<S> {
    pub fn is_initialized(&self) -> bool {
        match self.instrument_type {
            InstrumentType::Uninitialized => false,
            _ => true,
        }
    }

    pub fn is_expired_or_expiring(&self) -> bool {
        self.expiration_status == ExpirationStatus::Expired
            || self.expiration_status == ExpirationStatus::Expiring
    }

    pub fn is_expiring(&self) -> bool {
        self.expiration_status == ExpirationStatus::Expiring
    }

    pub fn is_fully_expired(&self) -> bool {
        self.expiration_status == ExpirationStatus::Expired
    }
}
