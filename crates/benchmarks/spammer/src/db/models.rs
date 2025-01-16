use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::spam_results)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SpamResObject {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub target_tps: i32,
    pub batch_submit_time: i32,
    pub error: bool
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::spam_results)]
pub struct NewSpamResObject {
    pub timestamp: NaiveDateTime,
    pub target_tps: i32,
    pub batch_submit_time: i32,
    pub error: bool
}
