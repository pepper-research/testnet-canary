use crate::MathError;
use crate::RiskError;
use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, num_derive::FromPrimitive, PartialEq)]
pub enum DexError {
    #[error("InvalidAccountData")]
    InvalidAccountData,
    #[error("DerivativeNotFound")]
    DerivativeNotFound,
    #[error("ContractIsExpired")]
    ContractIsExpired,
    #[error("ContractIsNotExpired")]
    ContractIsNotExpired,
    #[error("The given order index is invalid.")]
    InvalidOrderIndex,
    #[error("The user account has reached its maximum capacity for open orders.")]
    UserAccountFull,
    #[error("The transaction has been aborted.")]
    TransactionAborted,
    #[error("A required user account is missing.")]
    MissingUserAccount,
    #[error("The specified order has not been found.")]
    OrderNotFound,
    #[error("The operation is a no-op")]
    NoOp,
    #[error("The user account is still active")]
    UserAccountStillActive,
    #[error("Market is still active")]
    MarketStillActive,
    #[error("Market product group has no empty slot")]
    FullMarketProductGroup,
    #[error("Missing Market Product")]
    MissingMarketProduct,
    #[error("Invalid Withdrawal Amount")]
    InvalidWithdrawalAmount,
    #[error("Taker Trader has no product")]
    InvalidTakerTrader,
    #[error("Funds negative or fraction")]
    FundsError,
    #[error("Product is inactive")]
    InactiveProductError,
    #[error("Too many open orders")]
    TooManyOpenOrdersError,
    #[error("No more open orders")]
    NoMoreOpenOrdersError,
    #[error("Non zero price tick exponent")]
    NonZeroPriceTickExponentError,
    #[error("Duplicate product name")]
    DuplicateProductNameError,
    #[error("Invalid Risk Response")]
    InvalidRiskResponseError,
    #[error("Invalid Operation for Account Health")]
    InvalidAccountHealthError,
    #[error("Orderbook is empty")]
    OrderbookIsEmptyError,
    #[error("Combos not removed for expired product")]
    CombosNotRemoved,
    #[error("Trader risk group is not liquidable")]
    AccountNotLiquidable,
    #[error("Funding precision is more granular than the limit")]
    FundingPrecisionError,
    #[error("Product decimal precision error")]
    ProductDecimalPrecisionError,
    #[error("Expected product to be an outright product")]
    ProductNotOutright,
    #[error("Expected product to be a combo product")]
    ProductNotCombo,
    #[error("Risk engine returned an invalid social loss vector")]
    InvalidSocialLossCalculation,
    #[error("Risk engine returned invalid product indices in social loss vector")]
    ProductIndexMismatch,
    #[error("Invalid order ID")]
    InvalidOrderID,
    #[error("Invalid bytes for zero-copy deserialization")]
    InvalidBytesForZeroCopyDeserialization,
    #[error("Incorrect print trade size")]
    IncorrectPrintTradeSize,
    #[error("Incorrect print trade price")]
    IncorrectPrintTradePrice,
    #[error("Incorrect print trade side (initializer took same side)")]
    IncorrectPrintTradeSide,
    #[error("Incorrect print trade operator creator fees")]
    IncorrectPrintTradeOperatorCreatorFees,
    #[error("Incorrect print trade operator counterparty fees")]
    IncorrectPrintTradeOperatorCounterpartyFees,
    #[error("Invalid print trade operator fees")]
    InvalidPrintTradeOperatorFees,
    #[error("Deposit declined because the amount has exceeded the current total deposit limit")]
    DepositLimitExceeded,
    #[error("Withdraw declined because the amount has exceeded the current total withdraw limit")]
    WithdrawLimitExceeded,
    #[error("Attempt to set a negative deposit limit")]
    NegativeDepositLimit,
    #[error("Attempt to set a negative withdraw limit")]
    NegativeWithdrawLimit,
    #[error("update_product_funding called with 'Uninitialized' as the product status")]
    InvalidProductStatusInUpdateFunding,
    #[error("ContractIsNotExpiring")]
    ContractIsNotExpiring,
    #[error("Contract has non-zero open interest")]
    ContractHasNonZeroOpenInterest,
    #[error("Contract has non-zero open interest or risk state accounts")]
    ContractHasNonZeroOpenInterestOrRiskStateAccounts,
    #[error("ContractIsActive")]
    ContractIsActive,
    #[error("FailedToGetOrderQuantity")]
    FailedToGetOrderQuantity,
    #[error("SelfTradeBehaviorDecrementTakeIsDisallowed")]
    SelfTradeBehaviorDecrementTakeIsDisallowed,
    #[error("PriceBandViolation")]
    PriceBandViolation,
    #[error("Unexpected imbalanced open interest")]
    UnexpectedImbalancedOpenInterest,
    #[error("Maximum open interest exceeded")]
    MaximumOpenInterestExceeded,
    #[error("MarketProductGroupKillswitchIsOn")]
    MarketProductGroupKillswitchIsOn,
    #[error("InvalidFutureExpiry")]
    InvalidFutureExpiry,
    #[error("MaxReferrerFeeBpsExceeded")]
    MaxReferrerFeeBpsExceeded,
    #[error("PrintTradeOperatorDidNotSign")]
    PrintTradeOperatorDidNotSign,
    #[error("PrintTradeInvalidProductsLength")]
    PrintTradeInvalidProductsLength,
    #[error("ContractIsNotActive")]
    ContractIsNotActive,
    #[error("PrintTradeInvalidNumProducts")]
    PrintTradeInvalidNumProducts,
    #[error("PrintTradeProductMismatch")]
    PrintTradeProductMismatch,
    #[error("InsufficientLockedCollateral")]
    InsufficientLockedCollateral,
    #[error("OracleNotWhitelisted")]
    OracleNotWhitelisted,
    #[error("A negative liquidation price must equal total social loss")]
    NegativeLiquidationPriceDoesNotEqualSocialLoss,
    #[error("Negative social loss was seen; this should never happen")]
    NegativeSocialLoss,
    #[error("During social loss, total applicable shares was calculated to be negative; this should never happen"
    )]
    SocialLossNegativeTotalApplicableShares,
    #[error("MarketProductGroupAdminModeIsOn: everything disabled except cancel, consume")]
    MarketProductGroupAdminModeIsOn,
    #[error("Order match limit set to zero")]
    MatchLimitZero,
    #[error("Market product group does not exist")]
    MarketProductGroupDoesNotExist,
    #[error("Trader risk group does not exist")]
    TraderRiskGroupDoesNotExist,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ExchangeError {
    #[error("Math Error")]
    UtilErr(MathError),
    #[error("Dex Error")]
    DexErr(DexError),
    #[error("Risk Error")]
    RiskErr(RiskError),
}

#[derive(Error, Debug, Copy, Clone, FromPrimitive, PartialEq)]
pub enum UtilError {
    #[error("AccountAlreadyInitialized")]
    AccountAlreadyInitialized,
    #[error("AccountUninitialized")]
    AccountUninitialized,
    #[error("DuplicateProductKey")]
    DuplicateProductKey,
    #[error("PublicKeyMismatch")]
    PublicKeyMismatch,
    #[error("AssertionError")]
    AssertionError,
    #[error("InvalidMintAuthority")]
    InvalidMintAuthority,
    #[error("IncorrectOwner")]
    IncorrectOwner,
    #[error("PublicKeysShouldBeUnique")]
    PublicKeysShouldBeUnique,
    #[error("NotRentExempt")]
    NotRentExempt,
    #[error("NumericalOverflow")]
    NumericalOverflow,
    #[error("Rounding loses precision")]
    RoundError,
    #[error("Division by zero")]
    DivisionbyZero,
    #[error("Invalid return value")]
    InvalidReturnValue,
    #[error("Negative Number Sqrt")]
    SqrtRootError,
    #[error("Zero Price Error")]
    ZeroPriceError,
    #[error("Zero Quantity Error")]
    ZeroQuantityError,
    #[error("Serialization Error")]
    SerializeError,
    #[error("Deserialization Error")]
    DeserializeError,
    #[error("Invalid index for bitset")]
    InvalidBitsetIndex,
    #[error("Push failed because Bitvec is full")]
    PushToFullBitvec,
    #[error("Storing the value u8::MAX is not allowed in Bitvec")]
    U8MaxNotAllowedInBitvec,
}

pub type DexResult<T = ()> = Result<T, DexError>;
pub type ExchangeResult<T = ()> = Result<T, ExchangeError>;
