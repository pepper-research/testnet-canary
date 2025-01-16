use {num_derive::FromPrimitive, thiserror::Error};

#[derive(Clone, Debug, Error, FromPrimitive)]
pub enum AAOBError {
    #[error("This account is already initialized")]
    AlreadyInitialized,
    #[error("An invalid bids account has been provided.")]
    WrongBidsAccount,
    #[error("An invalid asks account has been provided.")]
    WrongAsksAccount,
    #[error("An invalid event queue account has been provided.")]
    WrongEventQueueAccount,
    #[error("An invalid caller authority account has been provided.")]
    WrongCallerAuthority,
    // #[error("Account type mismatch")]
    // AccountTypeMismatch,
    #[error("The event queue is full.")]
    EventQueueFull,
    #[error("The order could not be found.")]
    OrderNotFound,
    #[error("The order would self trade.")]
    WouldSelfTrade,
    #[error("The market's memory is full.")]
    SlabOutOfSpace,
    #[error("The due fee was not paid.")]
    FeeNotPaid,
    #[error("This instruction is a No-op.")]
    NoOperations,
    #[error("The market is still active")]
    MarketStillActive,
    #[error("The base quantity must be > 0")]
    InvalidBaseQuantity,
    #[error("The event queue should be owned by the AO program")]
    WrongEventQueueOwner,
    #[error("The bids account should be owned by the AO program")]
    WrongBidsOwner,
    #[error("The asks account should be owned by the AO program")]
    WrongAsksOwner,
    #[error("The market account should be owned by the AO program")]
    WrongMarketOwner,
    #[error("Limit price must be a tick size multiple")]
    InvalidLimitPrice,
    #[error("Numerical overlflow")]
    NumericalOverflow,
    #[error("Invalid callback info")]
    InvalidCallbackInfo,
    #[error("Account tag mismatch")]
    AccountTagMismatch,
}

pub type AAOBResult<T = ()> = Result<T, AAOBError>;
