use crate::{oracle::NUMBER_OF_MARKETS, Fractional};

#[cfg(feature = "offchain")]
use {
    super::models::NewPriceTick,
    crate::{db::utils::establish_connection, schema::price_tick},
    chrono::DateTime,
    diesel::RunQueryDsl,
};

#[cfg(feature = "offchain")]
pub fn insert_price_tick(product_index: i32, timestamp: u64, price: Fractional, confidence: u32) {
    let mut connection = establish_connection();

    let new_tick: NewPriceTick = NewPriceTick {
        product_index,
        timestamp: DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap()
            .naive_utc(),
        price: price.to_float(),
        confidence: confidence as i32,
    };

    diesel::insert_into(price_tick::table)
        .values(&new_tick)
        .execute(&mut connection)
        .expect("Error inserting price tick");
}

#[cfg(feature = "offchain")]
pub fn insert_price_ticks(
    // product_indices: Vec<i32>,
    timestamp: u64,
    prices: [Fractional; NUMBER_OF_MARKETS],
    confidences: [u32; NUMBER_OF_MARKETS],
) {
    let mut connection = establish_connection();

    let new_ticks: Vec<NewPriceTick> = (0..NUMBER_OF_MARKETS)
        .map(|i| NewPriceTick {
            product_index: i as i32,
            timestamp: DateTime::from_timestamp(timestamp as i64, 0)
                .unwrap()
                .naive_utc(),
            price: prices[i].to_float(),
            confidence: confidences[i] as i32,
        })
        .collect();

    diesel::insert_into(price_tick::table)
        .values(&new_ticks)
        .execute(&mut connection)
        .expect("Error inserting price ticks");
}
