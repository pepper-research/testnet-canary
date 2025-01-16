pub use {
    combo::*, outright::*, product::*, product_metadata::*, product_status::*, products_arrays::*,
};

use crate::aaob::OrderbookId;

pub mod combo;
pub mod outright;
pub mod product;
pub mod product_metadata;
pub mod product_status;
pub mod products_arrays;

use crate::ProductId;

// pub const NAME_LEN: usize = 16;
// pub const MAX_LEGS: usize = 4;

pub trait ProductTrait {
    fn get_product_key(&self) -> &ProductId;

    fn is_combo(&self) -> bool;

    fn get_name(&self) -> &[u8; 16];

    fn get_orderbook_id(&self) -> &OrderbookId;
}
