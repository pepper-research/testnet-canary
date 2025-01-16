use crate::state::RiskError;
use thiserror::Error;
use spicenet_shared::MathError;

// TODO: remove anchor-related error codes
#[derive(Error, Debug, Copy, Clone, num_derive::FromPrimitive, PartialEq)]
pub enum DexError {
    #[error("ContractIsExpired")]
    ContractIsExpired,
    #[error("ContractIsNotExpired")]
    ContractIsNotExpired,
    // #[error("Invalid system program account provided")]
    // InvalidSystemProgramAccount,
    // #[error("Invalid AOB program account provided")]
    // InvalidAobProgramAccount,
    // #[error("A provided state account was not owned by the current program")]
    // InvalidStateAccountOwner,
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
    // #[error("The user does not own enough lamports")]
    // OutofFunds,
    #[error("The user account is still active")]
    UserAccountStillActive,
    #[error("Market is still active")]
    MarketStillActive,
    // #[error("Invalid market signer provided")]
    // InvalidMarketSignerAccount,
    // #[error("Invalid orderbook account provided")]
    // InvalidOrderbookAccount,
    // #[error("Invalid market admin account provided")]
    // InvalidMarketAdminAccount,
    // #[error("Invalid base vault account provided")]
    // InvalidBaseVaultAccount,
    // #[error("Invalid quote vault account provided")]
    // InvalidQuoteVaultAccount,
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
    // #[error("Deposit declined due to insufficient balance on whitelist ATA token")]
    // DepositDeniedInsufficientBalanceOnWhitelistAtaToken,
    // #[error("Deposit declined due to unfrozen whitelist ATA token")]
    // DepositDeclinedUnfrozenWhitelistAtaToken,
    // #[error("Deposit declined due to non-existent whitelist ATA token on trader risk group")]
    // DepositDeclinedNonExistentWhitelistAtaTokenOnTraderRiskGroup,
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
    #[error("During social loss, total applicable shares was calculated to be negative; this should never happen")]
    SocialLossNegativeTotalApplicableShares,
    #[error("MarketProductGroupAdminModeIsOn: everything disabled except cancel, consume")]
    MarketProductGroupAdminModeIsOn,
    #[error("Order match limit set to zero")]
    MatchLimitZero,
}

pub type ExchangeResult<T = ()> = Result<T, ExchangeError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ExchangeError {
    UtilErr(MathError),
    DexErr(DexError), // temp, will be moved to `dex` module
    RiskErr(RiskError),
}

pub type DexResult<T = ()> = Result<T, DexError>;
pub type ExchangeResult<T = ()> = Result<T, ExchangeError>;