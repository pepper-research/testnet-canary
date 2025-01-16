use std::ops::{Deref, DerefMut};

use sov_modules_api::Spec;

use spicenet_shared::{Fractional, ZERO_FRAC};

use crate::state::dex::DexError;

use super::{ProductMetadata, ProductStatus};
use crate::state::ProductId;
use spicenet_aaob::OrderbookId;
// use crate::state::IsInitialized;
use crate::state::dex::ProductTrait;

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct OutrightProduct<S: Spec> {
    /// The associated metadata of the outright product.
    pub metadata: ProductMetadata<S>,

    /// The number of risk states, i.e parameters tracking open positions in this outright product.
    /// At 0, this means that no risk state is tracking positions(outstanding risk) in this outright product, which likely
    /// means that there are no positions present in the outright product.
    pub num_tracking_risk_states: usize,

    /// The status of the product.
    pub product_status: ProductStatus,

    /// TODO
    pub dust: Fractional,

    /// TODO
    pub cumulative_funding_per_share: Fractional,

    /// TODO
    pub cumulative_social_loss_per_share: Fractional,

    /// Open long positions opened on the exchange represented in notional value.
    pub open_long_interest: Fractional,

    /// Open short positions opened on the exchange represented in notional value.
    pub open_short_interest: Fractional,

    /// TODO
    pub mark_price_qualifying_cum_value: Fractional,

    /// TODO
    pub mark_price_max_qualifying_width: Fractional,
    pub padding: [u64; 10],
}

impl<S: Spec> OutrightProduct<S> {
    /// [`apply_new_funding()`] allows us to update cumulative funding per share with new funding values(represented by `amt_per_share`),
    /// followed by rounding it off to the number of decimals in cash.
    pub fn apply_new_funding(
        &mut self,
        amt_per_share: Fractional,
        cash_decimals: u64,
    ) -> Result<(), DexError> {
        self.cumulative_funding_per_share = (self.cumulative_funding_per_share + amt_per_share)
            .round_unchecked(cash_decimals as u32)?;

        Ok(())
    }

    /// [`apply_social_loss()`] allows us to update cumulative social loss per share with new social loss values(represented by `social_loss_per_share`)
    /// followed by rounding it off to the number of decimals in cash.
    pub fn apply_social_loss(
        &mut self,
        total_loss: Fractional,
        total_shares: Fractional,
        cash_decimals: u64,
    ) -> Result<(), DexError> {
        if total_shares > ZERO_FRAC {
            let social_loss_per_share = total_loss.checked_div(total_shares)?;

            self.cumulative_social_loss_per_share +=
                social_loss_per_share.round_unchecked(cash_decimals as u32)?;
        }

        Ok(())
    }

    /// We determine that an outright product is dormant if there is no open long and short interest(i.e both equal to ZERO_FRAC)
    pub fn is_dormant(&self) -> bool {
        self.open_long_interest == ZERO_FRAC && self.open_short_interest == ZERO_FRAC
    }

    /// We determine that an outright product is removable/settle-able if [`is_dormant()`] returns true AND there are no active risk states tracking the
    /// product at the moment.
    pub fn is_removable(&self) -> bool {
        self.is_dormant() && self.num_tracking_risk_states == 0
    }

    pub fn is_expired_or_expiring(&self) -> bool {
        self.product_status == ProductStatus::Expiring
            || self.product_status == ProductStatus::Expired
    }

    pub fn is_expired(&self) -> bool {
        self.product_status == ProductStatus::Expired
    }

    pub fn is_expiring(&self) -> bool {
        self.product_status == ProductStatus::Expiring
    }

    pub fn is_unitialized(&self) -> bool {
        self.product_status == ProductStatus::Uninitialized
    }

    /// TODO: need to make sense of this
    pub fn update_open_interest_change(
        &mut self,
        trade_size: Fractional,
        buyer_short_position: Fractional,
        seller_long_position: Fractional,
    ) -> Result<(), DexError> {
        match (
            buyer_short_position < trade_size,
            seller_long_position < trade_size,
        ) {
            (true, true) => {
                self.open_long_interest = self
                    .open_long_interest
                    .checked_add(trade_size)?
                    .checked_sub(buyer_short_position)?
                    .checked_sub(seller_long_position)?;
            }
            (true, false) => {
                self.open_long_interest =
                    self.open_long_interest.checked_sub(buyer_short_position)?;
            }
            (false, true) => {
                self.open_long_interest =
                    self.open_long_interest.checked_sub(seller_long_position)?;
            }
            (false, false) => {
                self.open_long_interest = self.open_long_interest.checked_sub(trade_size)?;
            }
        };
        self.open_short_interest = self.open_long_interest;
        Ok(())
    }
}

impl<S: Spec> Deref for OutrightProduct<S> {
    type Target = ProductMetadata<S>;

    fn deref(&self) -> &Self::Target {
        &self.metadata
    }
}

impl<S: Spec> DerefMut for OutrightProduct<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.metadata
    }
}

impl<S: Spec> ProductTrait for &OutrightProduct<S> {
    #[inline]
    fn get_product_key(&self) -> &ProductId {
        &self.metadata.product_id
    }

    #[inline]
    fn is_combo(&self) -> bool {
        false
    }

    #[inline]
    fn get_name(&self) -> &[u8; 16] {
        &self.metadata.name
    }

    #[inline]
    fn get_orderbook_id(&self) -> &OrderbookId {
        &self.metadata.orderbook_id
    }
}
