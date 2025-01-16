use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use spicenet_shared::dex::error::UtilError;

#[derive(BorshSerialize, BorshDeserialize, Copy, Debug, Clone, PartialEq)]
#[repr(u64)]
pub enum InstrumentType {
    Uninitialized,
    RecurringCall,
    RecurringPut,
    ExpiringCall,
    ExpiringPut,
}
impl Default for InstrumentType {
    fn default() -> Self {
        InstrumentType::Uninitialized
    }
}

impl InstrumentType {
    pub fn is_recurring(&self) -> std::result::Result<bool, UtilError> {
        match self {
            InstrumentType::RecurringCall | InstrumentType::RecurringPut => Ok(true),
            InstrumentType::ExpiringCall | InstrumentType::ExpiringPut => Ok(false),
            InstrumentType::Uninitialized => Err(UtilError::AccountUninitialized),
        }
    }
}

#[derive(
    BorshSerialize, BorshDeserialize, Copy, Debug, Clone, PartialEq, Serialize, Deserialize,
)]
#[repr(u64)]
pub enum OracleType {
    Uninitialized,
    LookupTable1,
}

impl Default for OracleType {
    fn default() -> Self {
        OracleType::Uninitialized
    }
}

#[derive(
    BorshSerialize, BorshDeserialize, Copy, Clone, Debug, PartialEq, Serialize, Deserialize,
)]
#[repr(u64)]
pub enum ExpirationStatus {
    Active,
    Expired,
    Expiring,
}
impl Default for ExpirationStatus {
    fn default() -> Self {
        ExpirationStatus::Active
    }
}
