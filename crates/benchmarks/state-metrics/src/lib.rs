use crate::db::models::{StateMetricsObjectDelta, StateMetricsObjectRaw};
use crate::db::samples::insert_state_metric;
use crate::db::utils::establish_connection;

pub mod db;
pub mod schema;

static mut prev_state_metrics: StateMetricsObjectRaw = StateMetricsObjectRaw {
    da_blocks_processed: 0.0,
    rollup_batches_processed: 0.0,
    batch_bytes_processed: 0.0,
    proof_blobs_processed: 0.0,
    proof_bytes_processed: 0.0,
    rollup_txns_processed: 0.0,
    rollup_txns_per_da_block: 0.0,
    current_da_height: 0.0,
    sync_distance: 0.0,
    process_slot_sec_sum: 0.0,
    stf_transition_sec_sum: 0.0,
    get_block_sec_sum: 0.0,
};

pub unsafe fn process_metrics(state_metric_obj: StateMetricsObjectRaw) {
    let mut connection = establish_connection();
    let delta_object = state_metric_obj.delta(&prev_state_metrics);
    println!("{:?}", delta_object.sync_distance);
    insert_state_metric(&mut connection, delta_object);
    prev_state_metrics = state_metric_obj;
}
