use std::fmt;
use sov_modules_api::impl_hash32_type;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    sov_modules_api::macros::UniversalWallet,
)]
#[schemars(bound = "S::Address: ::schemars::JsonSchema", rename = "MarketCallerAuthority")]
pub struct MarketCallerAuthority<S: ::sov_modules_api::Spec>(S::Address);

impl<S: ::sov_modules_api::Spec> MarketCallerAuthority<S> {
    #[doc = r" Public constructor"]
    pub fn new(address: &S::Address) -> Self {
        MarketCallerAuthority(address.clone())
    }
    #[doc = r" Public getter"]
    pub fn get_address(&self) -> &S::Address {
        &self.0
    }
}
impl<S: ::sov_modules_api::Spec> fmt::Display for MarketCallerAuthority<S>
where
    S::Address: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(core::format_args!("{}", self.0))
    }
}
impl<S: ::sov_modules_api::Spec> AsRef<[u8]> for MarketCallerAuthority<S>
where
    S::Address: AsRef<[u8]>,
{
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl_hash32_type!(MarketId, MarketIdBech32, "market");

impl PartialEq<MarketId> for &MarketId {
    fn eq(&self, other: &MarketId) -> bool {
        self.0 == other.0
    }
}
