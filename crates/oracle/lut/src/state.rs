use spicenet_shared::{Fractional, ZERO_FRAC};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone, Copy)]
pub struct ProductTick {
    pub price: Fractional,
    pub confidence: u32,
}

impl Default for ProductTick {
    fn default() -> Self {
        ProductTick {
            price: ZERO_FRAC,
            confidence: 0,
        }
    }
}
