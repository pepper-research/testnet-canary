#![no_main]
//! This binary implements the verification logic for the rollup. This is the code that runs inside
//! of the zkvm in order to generate proofs for the rollup.

use sov_mock_da::{MockDaSpec, MockDaVerifier};
pub use sov_mock_zkvm::{MockZkGuest, MockZkvm};
use sov_modules_api::default_spec::DefaultSpec;
use sov_modules_api::execution_mode::Zk;
use sov_modules_stf_blueprint::StfBlueprint;
use sov_sp1_adapter::guest::SP1Guest;
use sov_sp1_adapter::SP1;
use sov_state::ZkStorage;
use spicenet_stf::runtime::Runtime;
use spicenet_stf::StfVerifier;

sp1_zkvm::entrypoint!(main);

#[cfg_attr(feature = "bench", sov_cycle_utils::macros::cycle_tracker)]
pub fn main() {
    let guest = SP1Guest::new();
    let storage = ZkStorage::new();

    let stf: StfBlueprint<DefaultSpec<MockDaSpec, SP1, MockZkvm, Zk>, Runtime<_>> =
        StfBlueprint::new();

    let stf_verifier = StfVerifier::<_, _, _, _, _>::new(stf, MockDaVerifier {});

    stf_verifier
        .run_block(guest, storage)
        .expect("Prover must be honest");
}
