-- Your SQL goes here

CREATE TABLE "state_metrics"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"timestamp" TIMESTAMP NOT NULL,
	"da_blocks_processed" numeric NOT NULL,
	"rollup_batches_processed" numeric NOT NULL,
	"batch_bytes_processed" numeric NOT NULL,
	"proof_blobs_processed" numeric NOT NULL,
	"proof_bytes_processed" numeric NOT NULL,
	"rollup_txns_processed" numeric NOT NULL,
	"rollup_txns_per_da_block" numeric NOT NULL,
	"current_da_height" numeric NOT NULL,
	"sync_distance" numeric NOT NULL,
	"process_slot_sec" numeric NOT NULL,
	"stf_transition_sec" numeric NOT NULL,
	"get_block_sec" numeric NOT NULL
);

CREATE INDEX ON "state_metrics" ("timestamp");