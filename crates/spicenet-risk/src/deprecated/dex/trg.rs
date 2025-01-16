use sov_modules_api::{Address, Spec};

use crate::state::IsInitialized; // TODO: this shouldn't be imported from risk state, check for other occurences as well
use spicenet_shared::{addresses::TrgId, Fractional, MPGId, ProductId, Side, ZERO_FRAC, MAX_OUTRIGHTS, MAX_TRADER_POSITIONS};

use super::{AccountTag, DexError, DexResult, OpenOrders};

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct TraderRiskGroup<S: Spec> {
    pub tag: AccountTag,
    pub market_product_group: MPGId,
    pub id: TrgId<S>,
    // Default value is 255 (max int) which corresponds to no position for the product at the corresponding index
    pub active_products: [u8; MAX_OUTRIGHTS],
    pub total_deposited: Fractional,
    pub total_withdrawn: Fractional,
    // Treat cash separately since it is collateral (unless we eventually support spot)
    pub cash_balance: Fractional,
    // Keep track of pending fills for risk calculations (only for takers)
    pub pending_cash_balance: Fractional,
    // Keep track of pending taker fees to be collected in consume_events
    pub pending_fees: Fractional,
    pub valid_until: u32,
    pub maker_fee_bps: i32,
    pub taker_fee_bps: i32,
    pub trader_positions: [TraderPosition<S>; MAX_TRADER_POSITIONS],
    // pub risk_state_account: StateMap<TrgId<S>, VarianceCache<S>>,
    pub fee_state_account: Address<S>, // TODO: Dummy placeholder
    pub locked_collateral: [LockedCollateral; MAX_TRADER_POSITIONS], // in one-to-one mapping with trader_positions
    pub notional_maker_volume: Fractional,
    pub notional_taker_volume: Fractional,
    pub referred_takers_notional_volume: Fractional,
    /// referral_fees is not necessarily REFERRER_FEES_PROPORTION * referred_takers_notional_volume,
    /// because combo volume has only collects 1/8th the fees as outright volume
    pub referral_fees: Fractional,
    // unused
    pub allocated_for_future_use: [u8; 256],
    pub open_orders: OpenOrders,
}

impl<S: Spec> IsInitialized for TraderRiskGroup<S> {
    fn is_initialized(&self) -> bool {
        self.tag == AccountTag::TraderRiskGroup
    }
}

impl<S> TraderRiskGroup<S> {
    pub fn find_position_index(&self, product_id: &ProductId) -> Option<usize> {
        self.trader_positions
            .iter()
            .position(|pk| pk.product_key == *product_id) // Product id is the key stored in position
    }

    // Positions have is_initialized impl above, if it's true, adding it to total active positions
    pub fn num_active_positions(&self) -> usize {
        let mut num_active_positions = 0;
        for p in self.trader_positions {
            if !p.is_initialized() {
                continue;
            }
            num_active_positions += 1;
        }
        num_active_positions
    }

    // This may NOT work right now and would need rewrite after sokoban implementation
    pub fn remove_open_order_by_index(
        &mut self,
        product_index: usize,
        order_index: usize,
        order_id: u128,
    ) -> DexResult {
        // TODO: consider reinstating is_active check at some point
        let num_open_orders = self.open_orders.products[product_index].num_open_orders;
        // assert!(num_open_orders > 0, DexError::NoMoreOpenOrdersError.into())?;

        self.open_orders.products[product_index].num_open_orders -= 1;
        self.open_orders.total_open_orders = self.open_orders.total_open_orders.saturating_sub(1);
        self.open_orders
            .remove_open_order_by_index(product_index, order_index, order_id)
            .map_err(Into::into)
    }

    pub fn adjust_book_qty(
        &mut self,
        product_index: usize,
        qty: Fractional,
        side: Side,
        base_decimals: u64,
    ) -> DexResult {
        assert_eq!(qty.exp, base_decimals);

        let open_orders = &mut self.open_orders.products[product_index];

        match side {
            Side::Bid => {
                open_orders.bid_qty_in_book =
                    Fractional::new(open_orders.bid_qty_in_book, base_decimals)
                        .checked_add(qty)?
                        .round(base_decimals as u32)?
                        .m;
            }
            Side::Ask => {
                open_orders.ask_qty_in_book =
                    Fractional::new(open_orders.ask_qty_in_book, base_decimals)
                        .checked_add(qty)?
                        .round(base_decimals as u32)?
                        .m;
            }
        }
        Ok(())
    }

    // reset_book_qty was used to fix bad state in trgs caused by not launching DEX and AAOB atomically
    pub fn reset_book_qty(&mut self, product_index: usize) -> DexResult {
        let open_orders = &mut self.open_orders.products[product_index];
        // assert(open_orders.num_open_orders == 0, DexError::ContractIsActive)?;
        open_orders.num_open_orders = 0;
        open_orders.bid_qty_in_book = 0;
        open_orders.ask_qty_in_book = 0;
        Ok(())
    }

    pub fn decrement_book_size(
        &mut self,
        product_index: usize,
        side: Side,
        qty: Fractional,
        base_decimals: u64,
    ) -> DexResult {
        let open_orders = &mut self.open_orders.products[product_index];

        match side {
            Side::Bid => {
                open_orders.bid_qty_in_book =
                    Fractional::new(open_orders.bid_qty_in_book, base_decimals)
                        .checked_sub(qty)?
                        .round(base_decimals as u32)?
                        .m;
            }
            Side::Ask => {
                open_orders.ask_qty_in_book =
                    Fractional::new(open_orders.ask_qty_in_book, base_decimals)
                        .checked_sub(qty)?
                        .round(base_decimals as u32)?
                        .m;
            }
        }
        Ok(())
    }

    pub fn decrement_order_size_by_index(&mut self, order_index: usize, qty: u64) -> DexResult {
        self.open_orders
            .decrement_order_size_by_index(order_index, qty)
    }

    pub fn decrement_order_size(
        &mut self,
        product_index: usize,
        order_id: u128,
        qty: u64,
    ) -> DexResult {
        self.open_orders
            .decrement_order_size(product_index, order_id, qty)
    }

    pub fn is_active_product(&self, index: usize) -> std::result::Result<bool, DexResult> {
        if !self.is_initialized() {
            // msg!("TraderRiskGroup is not initialized");
            return Err(DexError::InvalidAccountData.into());
        }
        if index >= MAX_OUTRIGHTS {
            // msg!(
            //     "Product index is out of bounds. index: {}, max products: {}",
            //     index,
            //     MAX_OUTRIGHTS
            // );
            return Err(DexError::InvalidAccountData.into());
        }
        Ok(self.active_products[index] != u8::MAX)
    }

    // maps product index in mpg to trader position index
    pub fn get_position_index(&self, index: usize) -> std::result::Result<u8, DexResult> {
        if !self.is_initialized() {
            // msg!("TraderRiskGroup is not initialized");
            return Err(DexError::InvalidAccountData.into());
        }
        if index >= MAX_OUTRIGHTS {
            // msg!(
            //     "Product index is out of bounds. index: {}, max products: {}",
            //     index,
            //     MAX_OUTRIGHTS
            // );
            return Err(DexError::InvalidAccountData.into());
        }
        Ok(self.active_products[index])
    }

    pub fn clear(&mut self, product_id: &ProductId) -> DexResult {
        let trader_position_index = match self.find_position_index(product_id) {
            Some(i) => i,
            None => {
                return Err(DexError::InvalidAccountData.into());
            }
        };
        let trader_position = &mut self.trader_positions[trader_position_index];
        self.active_products[trader_position.product_index] = u8::MAX;
        trader_position.tag = AccountTag::Uninitialized;
        // trader_position.product_key = Pubkey::default(); // The reason we don't zero this is for the risk engine.
        trader_position.position = ZERO_FRAC;
        trader_position.pending_position = ZERO_FRAC;
        trader_position.product_index = 0;
        trader_position.last_cum_funding_snapshot = ZERO_FRAC;
        trader_position.last_social_loss_snapshot = ZERO_FRAC;
        let locked_collateral = &mut self.locked_collateral[trader_position_index]; // use trader_position_index because one-to-one
        locked_collateral.tag = AccountTag::Uninitialized;
        locked_collateral.ask_qty = ZERO_FRAC;
        locked_collateral.bid_qty = ZERO_FRAC;
        Ok(())
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct TraderPosition<S: Spec> {
    pub tag: AccountTag,
    pub product_key: ProductId,
    pub position: Fractional,
    pub pending_position: Fractional,
    pub product_index: usize,
    pub last_cum_funding_snapshot: Fractional,
    pub last_social_loss_snapshot: Fractional,
}

impl<S: Spec> IsInitialized for TraderPosition<S> {
    fn is_initialized(&self) -> bool {
        self.tag == AccountTag::TraderPosition
    }
}

impl<S: Spec> TraderPosition<S> {
    pub fn is_active(&self) -> bool {
        self.position != ZERO_FRAC || self.pending_position != ZERO_FRAC
    }
}

#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(schemars::JsonSchema)
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct LockedCollateral {
    pub tag: AccountTag,
    pub ask_qty: Fractional,
    pub bid_qty: Fractional,
}

impl IsInitialized for LockedCollateral {
    fn is_initialized(&self) -> bool {
        self.tag == AccountTag::LockedCollateral
    }
}

impl LockedCollateral {
    pub const MAX_PRODUCTS_PER_LOCK_IX: usize = 6;

    pub fn default() -> Self {
        LockedCollateral {
            tag: AccountTag::Uninitialized,
            ask_qty: ZERO_FRAC,
            bid_qty: ZERO_FRAC,
        }
    }
}

#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Clone, Debug, PartialEq, Eq)]
pub struct LockedCollateralProductIndex {
    pub product_index: usize,
    pub size: Fractional,
}

pub type LockedCollateralProductIndexes =
    [LockedCollateralProductIndex; LockedCollateral::MAX_PRODUCTS_PER_LOCK_IX];

impl LockedCollateral {
    pub const MAX_PRODUCTS_PER_LOCK_IX: usize = 6;
    pub fn default() -> Self {
        LockedCollateral {
            tag: AccountTag::Uninitialized,
            // From shared crate
            ask_qty: ZERO_FRAC,
            bid_qty: ZERO_FRAC,
        }
    }
}
