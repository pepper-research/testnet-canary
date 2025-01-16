use crate::Fractional;

use crate::CallBackInfo;

pub const NAME_LEN: usize = 16;

pub const MAX_OUTRIGHTS: usize = 128;

pub const MAX_PRODUCTS: usize = 256;

pub const HEALTH_BUFFER_LEN: usize = 32;

pub const MAX_TRADER_POSITIONS: usize = 16;

pub const MAX_OPEN_ORDERS_PER_POSITION: u64 = 64;

pub const MAX_OPEN_ORDERS: usize = 100;

pub const ANCHOR_DISCRIMINANT_LEN: usize = 8;

pub const EVENTS_NUM_DECIMALS: u32 = 8;

pub const NO_BID_PRICE: Fractional = Fractional {
    m: i64::MIN,
    exp: 0,
};

pub const NO_ASK_PRICE: Fractional = Fractional {
    m: i64::MAX,
    exp: 0,
};

pub const SENTINEL: u16 = 0;

/// The length in bytes of the callback information in the associated asset agnostic orderbook

// pub const CALLBACK_INFO_LEN: u64 = std::mem::size_of::<CallBackInfo> as u64; //todo: figure out a way to implement spec here or use formula directly instead of constant
/// The length in bytes of the callback identifer prefix in the associated asset agnostic orderbook

pub const CALLBACK_ID_LEN: u64 = 32;

pub const MAX_COMBOS: usize = 128;

pub const MAX_LEGS: usize = 4;

// timing constants

pub const SLOTS_1_MIN: u64 = 150;

pub const SLOTS_5_MIN: u64 = 750;

pub const SLOTS_15_MIN: u64 = 2250;

pub const SLOTS_60_MIN: u64 = 9000;

pub const COMBO_FEE_SCALAR: Fractional = Fractional { m: 125, exp: 3 };
