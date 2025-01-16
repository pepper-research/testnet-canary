use thiserror::Error;

pub type MathResult<T> = Result<T, MathError>;

#[derive(Error, Debug, Clone, Copy, num_derive::FromPrimitive, PartialEq)]
pub enum MathError {
    #[error("NumericalOverflow")]
    NumericalOverflow,
    #[error("DeserializationError")]
    DeserializationError,
    #[error("RoundError")]
    RoundError,
    #[error("SqrtError")]
    SqrtError,
    #[error("DivisionByZero")]
    DivisionByZero,
    #[error("FastIntCoercionToZero")]
    FastIntCoercionToZero,
    #[error("InvalidBitPairIndex")]
    InvalidBitPairIndex,
}
