use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, FromPrimitive, PartialEq)]
pub enum DerivativeError {
    #[error("AccountAlreadyInitialized")]
    AccountAlreadyInitialized,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("InvalidSettlementTime")]
    InvalidSettlementTime,
    #[error("InvalidCreationTime")]
    InvalidCreationTime,
    #[error("UninitializedAccount")]
    UninitializedAccount,
    #[error("InvalidSequenceNumber")]
    InvalidSequenceNumber,
    #[error("UnsettledAccounts")]
    UnsettledAccounts,
    #[error("InvalidOracleConfig")]
    InvalidOracleConfig,
    #[error("NumericalOverflow")]
    NumericalOverflow,
    #[error("CannotBeDeleted")]
    CannotBeDeleted,
    #[error("ContractIsExpired")]
    ContractIsExpired,
    #[error("InvalidDate")]
    InvalidDate,
    #[error("InvalidAccount")]
    InvalidAccount,
    #[error("Unimplemented")]
    Unimplemented,
    #[error("FailedToReadOracle")]
    FailedToReadOracle,
    #[error("GetPriceError")]
    GetPriceError,
    #[error("InsufficientVerificationLevel")]
    InsufficientVerificationLevel,
    #[error("StaleOraclePrice")]
    StaleOraclePrice,
}
