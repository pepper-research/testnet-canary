#[cfg(feature = "offchain")]
use chrono::NaiveDateTime;
#[cfg(feature = "offchain")]
use diesel::prelude::*;

#[cfg(feature = "offchain")]
use diesel_derive_enum::DbEnum;

#[cfg(feature = "offchain")]
#[derive(DbEnum, Debug, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::LogLevel"]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

#[cfg(feature = "offchain")]
#[derive(DbEnum, Debug, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::LogScope"]
pub enum LogScope {
    Other,
    Risk,
    Dex,
    Aaob,
    Instruments,
    None,
}

#[cfg(feature = "offchain")]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Log {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub level: LogLevel,
    pub scope: LogScope,
    pub message: String,
}

#[cfg(feature = "offchain")]
#[derive(Insertable)]
#[diesel(table_name = crate::schema::log)]
pub struct NewLog {
    pub timestamp: NaiveDateTime,
    pub level: LogLevel,
    pub scope: LogScope,
    pub message: String,
}

#[cfg(feature = "offchain")]
#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::price_tick)]
pub struct PriceTick {
    pub id: i32,
    pub product_index: i32,
    pub timestamp: NaiveDateTime,
    pub price: f64,
    pub confidence: u32,
}
#[cfg(feature = "offchain")]
#[derive(Insertable)]
#[diesel(table_name = crate::schema::price_tick)]
pub struct NewPriceTick {
    pub product_index: i32,
    pub timestamp: NaiveDateTime,
    pub price: f64,
    pub confidence: i32,
}
