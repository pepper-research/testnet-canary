use sov_modules_api::digest::Digest;
use sov_modules_api::CryptoSpec;

use spicenet_shared::addresses::TrgId;

/// Derives trg ID from `trg_owner`
pub fn get_trg_id<S: sov_modules_api::Spec>(trg_owner: &str) -> TrgId<S> {
    let mut hasher = <S::CryptoSpec as CryptoSpec>::Hasher::new();
    hasher.update(trg_owner.as_bytes());
    let hash: [u8; 32] = hasher.finalize().into();
    hash.into()
}
