use crate::address::MarketId;
use sov_modules_api::digest::Digest;
use sov_modules_api::CryptoSpec;

/// Derives market ID from `market_name`, can be changed to some other seed too
pub fn get_market_id<S: sov_modules_api::Spec>(market_name: &str) -> MarketId {
    let mut hasher = <S::CryptoSpec as CryptoSpec>::Hasher::new();
    hasher.update(market_name.as_bytes());
    let hash: [u8; 32] = hasher.finalize().into();
    hash.into()
}

/// a is fp0, b is fp32 and result is a/b fp0
pub(crate) fn fp32_div(a: u64, b_fp32: u64) -> u64 {
    (((a as u128) << 32) / (b_fp32 as u128)) as u64
}

/// a is fp0, b is fp32 and result is a*b fp0
pub(crate) fn fp32_mul(a: u64, b_fp32: u64) -> u64 {
    (((a as u128) * (b_fp32 as u128)) >> 32) as u64
}
