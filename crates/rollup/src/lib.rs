
pub mod mock_rollup;

pub mod celestia_rollup;

/// ASCII "spicenet-b" (b stands for batch)
pub const ROLLUP_BATCH_NAMESPACE_RAW: [u8; 10] = [115, 112, 105, 99, 101, 110, 101, 116, 45, 98];

/// ASCII of "spicenet-p" (p stands for proof)
pub const ROLLUP_PROOF_NAMESPACE_RAW: [u8; 10] = [115, 112, 105, 99, 101, 110, 101, 116, 45, 112];

use sov_celestia_adapter::types::Namespace;

pub const ROLLUP_BATCH_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_BATCH_NAMESPACE_RAW);
pub const ROLLUP_PROOF_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_PROOF_NAMESPACE_RAW);