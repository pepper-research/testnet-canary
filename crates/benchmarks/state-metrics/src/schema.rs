// @generated automatically by Diesel CLI.

diesel::table! {
    state_metrics (id) {
        id -> Int4,
        timestamp -> Timestamp,
        da_blocks_processed -> Numeric,
        rollup_batches_processed -> Numeric,
        batch_bytes_processed -> Numeric,
        proof_blobs_processed -> Numeric,
        proof_bytes_processed -> Numeric,
        rollup_txns_processed -> Numeric,
        rollup_txns_per_da_block -> Numeric,
        current_da_height -> Numeric,
        sync_distance -> Numeric,
        process_slot_sec -> Numeric,
        stf_transition_sec -> Numeric,
        get_block_sec -> Numeric,
    }
}
