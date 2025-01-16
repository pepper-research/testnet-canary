use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
pub enum RiskError {
    // 1. parsed an unexpected account tag (expected tag to match initialized account)
    #[error("InvalidAccountTag")]
    InvalidAccountTag,

    // 2. parsed an unexpected account tag while initializing account
    #[error("AccountAlreadyInitialized")]
    AccountAlreadyInitialized,

    // 3. the risk signer account address (PDA) didn't match the expected account address
    #[error("InvalidRiskSigner")]
    InvalidRiskSigner,

    // 4. unexpected account owner
    #[error("InvalidAccountOwner")]
    InvalidAccountOwner,

    // 5. the input account address did mot match the expected address
    #[error("InvalidAccountAddress")]
    InvalidAccountAddress,

    // the expected authority to write to the covariance data was not supplied
    #[error("InvalidAuthority")]
    InvalidAuthority,

    // the program attempted to access an invalid location in the covariance matrix
    #[error("InvalidCovarianceMatrixAccess")]
    InvalidCovarianceMatrixAccess,

    // a necessary product key was not present in the existing covariance matrix
    #[error("MissingCovarianceEntry")]
    MissingCovarianceEntry,

    // invalid atttempt to take the square root of a negative number
    #[error("InvalidSqrtInput")]
    InvalidSqrtInput,

    // input to populate the covariance matrix accounts was invalid
    #[error("InvalidCovarianceInput")]
    InvalidCovarianceInput,

    // 10. a necessary instrument was missing bid, ask, previous bid, and previous ask information
    // necessary for the risk calculation
    #[error("MissingBBOForMarkPrice")]
    MissingBBOForMarkPrice,

    // calculation resulted in an overlfow of the internal numerical types
    #[error("NumericalOverflow")]
    NumericalOverflow,

    // attempted to access an unexpected product type (combo vs. outright)
    #[error("UnexpectedProductType")]
    UnexpectedProductType,

    // an internal assumption was violated
    #[error("UnexpectedResult")]
    UnexpectedResult,

    // the risk state account didn't match the account indicated in the trader risk group
    #[error("MismatchedRiskStateAccount")]
    MismatchedRiskStateAccount,

    // 15. failed to find the variance cache indices for all the legs of a combo
    #[error("FailedToFindCacheIndexForLeg")]
    FailedToFindCacheIndexForLeg,

    // this should never happen
    #[error("ComboSizeGreaterThanCollectionLen")]
    ComboSizeGreaterThanCollectionLen,

    // number of oracle price accounst must be even and in the format
    // [n oracle accounts] followed by [n derivative metadata accounts]
    // where the i-th oracle account corresponds to the i-th derivative metadata account
    #[error("InvalidMarkPriceAccountsLen")]
    InvalidMarkPriceAccountsLen,

    // the i-th oracle account is not the same as [the i-th derivative metadata account].price_oracle
    #[error("MismatchedOraclePriceAccount")]
    MismatchedOraclePriceAccount,

    // no entry in mark prices array corresponding to given product key
    #[error("MissingMarkPrice")]
    MissingMarkPrice,

    // 20. passed in MarkPricesArray PDA has incorrect bump
    #[error("IncorrectMarkPricesBump")]
    IncorrectMarkPricesBump,

    // 21. no empty slots in mark prices array
    #[error("MarkPricesArrayIsFull")]
    MarkPricesArrayIsFull,

    // 22. mark_prices.update_slot < current_slot
    #[error("MarkPricesOutOfDate")]
    MarkPricesOutOfDate,

    // 22. Failed to find market product index to in variance cache
    #[error("Failed to find market product index to in variance cache")]
    FailedToFindMarketProductIndexInVarianceCache,

    #[error("BookSpreadTooWideForMarkPrice")]
    BookSpreadTooWideForMarkPrice,

    #[error("PriceBandViolation")]
    PriceBandViolation,

    #[error("FastIntCoercionToZero")]
    FastIntCoercionToZero,

    #[error("InvalidRiskCheckParameters")]
    InvalidRiskCheckParameters,

    #[error("CovarianceMatrixAlreadyInitialized")]
    CovarianceMatrixAlreadyInitialized,

    #[error("CovarianceMatrixNotInitialized")]
    CovarianceMatrixNotInitialized,

    #[error("MarkPricesAlreadyInitialized")]
    MarkPricesAlreadyInitialized,

    #[error("MarkPricesNotInitalized")]
    MarkPricesNotInitialized,

    #[error("VarianceCacheAlreadyInitialized")]
    VarianceCacheAlreadyInitialized,

    #[error("VarianceCacheNotInitialized")]
    VarianceCacheNotInitialized,
}
