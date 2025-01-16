// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "log_level"))]
    pub struct LogLevel;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "log_scope"))]
    pub struct LogScope;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LogLevel;
    use super::sql_types::LogScope;

    log (id) {
        id -> Int4,
        timestamp -> Timestamp,
        level -> LogLevel,
        message -> Text,
        scope -> LogScope,
    }
}

diesel::table! {
    price_tick (id) {
        id -> Int4,
        product_index -> Int4,
        timestamp -> Timestamp,
        price -> Float8,
        confidence -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(log, price_tick,);
