use {
    borsh::{BorshDeserialize, BorshSerialize}, // num_traits::{FromPrimitive, ToBytes, ToPrimitive},
};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
)]
#[derive(BorshDeserialize, BorshSerialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
#[repr(u8)]
pub enum StateType {
    Uninitialized,
    Market,
    Bids,
    Asks,
}
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(Debug, BorshDeserialize, BorshSerialize, Clone, PartialEq, num_derive::FromPrimitive, Eq)]
pub enum SelfTradeHandler {
    /// The orders are matched together
    DecrementTake,
    /// the order on the maker side is cancelled. matching for the taker order continues and essentially bypasses the self-maker order
    CancelProvide,
    /// the entire transaction(taker side) fails with a program error
    AbortTx,
}

#[repr(u8)]
#[allow(missing_docs)]
pub enum CompletedReason {
    Cancelled,
    Filled,
    Booted,
    SelfTradeAbort,
    PostNotAllowed,
    MatchLimitExhausted,
    PostOnly,
}
