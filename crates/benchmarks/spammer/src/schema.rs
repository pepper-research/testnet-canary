// @generated automatically by Diesel CLI.

diesel::table! {
    spam_results (id) {
        id -> Integer,
        timestamp -> Timestamp,
        target_tps -> Integer,
        batch_submit_time -> Integer,
        error -> Bool,
    }
}
