use super::models::{NewStateMetricObject, StateMetricObject, StateMetricsObjectDelta};
use crate::schema::state_metrics;
use bigdecimal::BigDecimal;
use bigdecimal::FromPrimitive;
use chrono::NaiveDateTime;
use diesel::{Connection, PgConnection, Queryable, RunQueryDsl, Selectable, SelectableHelper};
use std::env;

pub fn insert_state_metric(
    connection: &mut PgConnection,
    state_metric_obj: StateMetricsObjectDelta,
) -> StateMetricObject {
    let new_sys_sample_object = NewStateMetricObject {
        timestamp: chrono::Utc::now().naive_utc(),
        da_blocks_processed: BigDecimal::from_f64(state_metric_obj.da_blocks_processed).unwrap(),
        rollup_batches_processed: BigDecimal::from_f64(state_metric_obj.rollup_batches_processed)
            .unwrap(),
        batch_bytes_processed: BigDecimal::from_f64(state_metric_obj.batch_bytes_processed)
            .unwrap(),
        proof_blobs_processed: BigDecimal::from_f64(state_metric_obj.proof_blobs_processed)
            .unwrap(),
        proof_bytes_processed: BigDecimal::from_f64(state_metric_obj.proof_bytes_processed)
            .unwrap(),
        rollup_txns_processed: BigDecimal::from_f64(state_metric_obj.rollup_txns_processed)
            .unwrap(),
        rollup_txns_per_da_block: BigDecimal::from_f64(state_metric_obj.rollup_txns_per_da_block)
            .unwrap(),
        current_da_height: BigDecimal::from_f64(state_metric_obj.current_da_height).unwrap(),
        sync_distance: BigDecimal::from_f64(state_metric_obj.sync_distance).unwrap(),
        process_slot_sec: BigDecimal::from_f64(state_metric_obj.process_slot_sec).unwrap(),
        stf_transition_sec: BigDecimal::from_f64(state_metric_obj.stf_transition_sec).unwrap(),
        get_block_sec: BigDecimal::from_f64(state_metric_obj.get_block_sec).unwrap(),
    };

    diesel::insert_into(state_metrics::table)
        .values(&new_sys_sample_object)
        .returning(StateMetricObject::as_returning())
        .get_result(connection)
        .expect("Error inserting metric")
}
