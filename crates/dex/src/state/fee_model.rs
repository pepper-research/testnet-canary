use sov_modules_api::Spec;
use spicenet_shared::{dex::MarketProductGroup, Fractional, ProductId, Side};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct TraderFees {
    pub valid_until: u32,
    pub maker_fee_bps: i32,
    pub taker_fee_bps: i32,
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct TraderFeeParams<S: Spec> {
    pub side: Side,
    pub is_aggressor: bool,
    pub matched_quote_qty: Fractional,
    pub matched_base_qty: Fractional,
    pub product: ProductId,
}

// 10_000 bps == 100%
const MAX_FEE_BPS: i32 = 10_000;
const MIN_FEE_BPS: i32 = -10_000;

fn clamp_fees(fee: i32) -> i32 {
    within_or_zero(fee, MAX_FEE_BPS, MIN_FEE_BPS)
}

fn within_or_zero(x: i32, max: impl Into<i32>, min: impl Into<i32>) -> i32 {
    if x > max.into() || x < min.into() {
        0
    } else {
        x
    }
}

impl TraderFees {
    pub fn new(maker_fee_bps: i32, taker_fee_bps: i32, valid_until: u32) -> Self {
        // valid_until is timestamp
        Self {
            valid_until,
            maker_fee_bps,
            taker_fee_bps,
        }
    }

    pub fn maker_fee_bps<S: Spec>(
        &self,
        market_product_group: Option<&MarketProductGroup<S>>,
    ) -> Fractional {
        let fee = market_product_group
            .map(|mpg| {
                within_or_zero(
                    self.maker_fee_bps,
                    mpg.max_maker_fee_bps,
                    mpg.min_maker_fee_bps,
                )
            })
            .unwrap_or(clamp_fees(self.maker_fee_bps));

        Fractional::new(fee as i64, 4) // bps
    }

    pub fn taker_fee_bps<S: Spec>(
        &self,
        market_product_group: Option<&MarketProductGroup<S>>,
    ) -> Fractional {
        let fee = market_product_group
            .map(|mpg| {
                within_or_zero(
                    self.taker_fee_bps,
                    mpg.max_taker_fee_bps,
                    mpg.min_taker_fee_bps,
                )
            })
            .unwrap_or(clamp_fees(self.taker_fee_bps));

        Fractional::new(fee as i64, 4) // bps
    }

    pub fn set_taker_fee_bps(&mut self, taker_fee_bps: i32) {
        self.taker_fee_bps = clamp_fees(taker_fee_bps);
    }

    pub fn set_maker_fee_bps(&mut self, maker_fee_bps: i32) {
        self.maker_fee_bps = clamp_fees(maker_fee_bps);
    }
}
