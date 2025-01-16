use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::samples)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SampleObject {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub nonce_latency: BigDecimal,
    pub publish_batch_latency: BigDecimal,
    pub ping_latency: BigDecimal,
    pub confirmation_latency: BigDecimal,
    pub e2e_latency: BigDecimal,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::samples)]
pub struct NewSampleObject {
    pub timestamp: NaiveDateTime,
    pub nonce_latency: BigDecimal,
    pub publish_batch_latency: BigDecimal,
    pub ping_latency: BigDecimal,
    pub confirmation_latency: BigDecimal,
    pub e2e_latency: BigDecimal,
}
