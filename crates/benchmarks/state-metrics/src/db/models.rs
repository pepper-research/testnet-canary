use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Debug)]
pub struct StateMetricsObjectRaw {
    pub da_blocks_processed: f64,
    pub rollup_batches_processed: f64,
    pub batch_bytes_processed: f64,
    pub proof_blobs_processed: f64,
    pub proof_bytes_processed: f64,
    pub rollup_txns_processed: f64,
    pub rollup_txns_per_da_block: f64,
    pub current_da_height: f64,
    pub sync_distance: f64,
    pub process_slot_sec_sum: f64,
    pub stf_transition_sec_sum: f64,
    pub get_block_sec_sum: f64,
}

pub struct StateMetricsObjectDelta {
    pub da_blocks_processed: f64,
    pub rollup_batches_processed: f64,
    pub batch_bytes_processed: f64,
    pub proof_blobs_processed: f64,
    pub proof_bytes_processed: f64,
    pub rollup_txns_processed: f64,
    pub rollup_txns_per_da_block: f64,
    pub current_da_height: f64,
    pub sync_distance: f64,
    pub process_slot_sec: f64,
    pub stf_transition_sec: f64,
    pub get_block_sec: f64,
}

impl StateMetricsObjectRaw {
    pub fn new() -> Self {
        StateMetricsObjectRaw {
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
        }
    }

    pub fn verify(&self) -> bool {
        self.da_blocks_processed > 0.0
            && self.rollup_batches_processed > 0.0
            && self.batch_bytes_processed > 0.0
            && self.proof_blobs_processed > 0.0
            && self.proof_bytes_processed > 0.0
            && self.rollup_txns_processed > 0.0
            && self.rollup_txns_per_da_block > 0.0
            && self.current_da_height > 0.0
            && self.sync_distance > 0.0
            && self.process_slot_sec_sum > 0.0
            && self.stf_transition_sec_sum > 0.0
            && self.get_block_sec_sum > 0.0
    }

    pub fn delta(&self, previous: &StateMetricsObjectRaw) -> StateMetricsObjectDelta {
        StateMetricsObjectDelta {
            da_blocks_processed: self.da_blocks_processed - previous.da_blocks_processed,
            rollup_batches_processed: self.rollup_batches_processed
                - previous.rollup_batches_processed,
            batch_bytes_processed: self.batch_bytes_processed - previous.batch_bytes_processed,
            proof_blobs_processed: self.proof_blobs_processed - previous.proof_blobs_processed,
            proof_bytes_processed: self.proof_bytes_processed - previous.proof_bytes_processed,
            rollup_txns_processed: self.rollup_txns_processed - previous.rollup_txns_processed,
            rollup_txns_per_da_block: self.rollup_txns_per_da_block,
            current_da_height: self.current_da_height,
            sync_distance: self.sync_distance,
            process_slot_sec: self.process_slot_sec_sum - previous.process_slot_sec_sum,
            stf_transition_sec: self.stf_transition_sec_sum - previous.stf_transition_sec_sum,
            get_block_sec: self.get_block_sec_sum - previous.get_block_sec_sum,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::state_metrics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StateMetricObject {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub da_blocks_processed: BigDecimal,
    pub rollup_batches_processed: BigDecimal,
    pub batch_bytes_processed: BigDecimal,
    pub proof_blobs_processed: BigDecimal,
    pub proof_bytes_processed: BigDecimal,
    pub rollup_txns_processed: BigDecimal,
    pub rollup_txns_per_da_block: BigDecimal,
    pub current_da_height: BigDecimal,
    pub sync_distance: BigDecimal,
    pub process_slot_sec: BigDecimal,
    pub stf_transition_sec: BigDecimal,
    pub get_block_sec: BigDecimal,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::state_metrics)]
pub struct NewStateMetricObject {
    pub timestamp: NaiveDateTime,
    pub da_blocks_processed: BigDecimal,
    pub rollup_batches_processed: BigDecimal,
    pub batch_bytes_processed: BigDecimal,
    pub proof_blobs_processed: BigDecimal,
    pub proof_bytes_processed: BigDecimal,
    pub rollup_txns_processed: BigDecimal,
    pub rollup_txns_per_da_block: BigDecimal,
    pub current_da_height: BigDecimal,
    pub sync_distance: BigDecimal,
    pub process_slot_sec: BigDecimal,
    pub stf_transition_sec: BigDecimal,
    pub get_block_sec: BigDecimal,
}
