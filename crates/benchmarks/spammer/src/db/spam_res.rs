use super::models::{NewSpamResObject, SpamResObject};
use bigdecimal::BigDecimal;
use crate::schema::spam_results;
use chrono::NaiveDateTime;
use diesel::{Connection, SqliteConnection, Queryable, RunQueryDsl, Selectable, SelectableHelper};
use std::env;
use crate::db::spam_res;

pub fn insert_res(connection: &mut SqliteConnection, target_tps: i32, batch_submit_time: i32, error: bool) -> SpamResObject {
    let new_sample_object = NewSpamResObject {
        timestamp: chrono::Utc::now().naive_utc(),
        target_tps,
        batch_submit_time,
        error,
    };

    diesel::insert_into(spam_results::table)
        .values(&new_sample_object).get_result(connection)
        .expect("Error inserting sample")
}
