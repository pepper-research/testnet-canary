// @generated automatically by Diesel CLI.

diesel::table! {
    samples (id) {
        id -> Int4,
        timestamp -> Timestamp,
        ping_latency -> Numeric,
        nonce_latency -> Numeric,
        publish_batch_latency -> Numeric,
        confirmation_latency -> Numeric,
        e2e_latency -> Numeric,
    }
}
