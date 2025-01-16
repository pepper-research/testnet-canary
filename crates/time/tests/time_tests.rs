#![allow(dead_code)]
use std::convert::Infallible;

use spicenet_time::{CallMessage, Event, TimeModule};

use sov_modules_api::prelude::UnwrapInfallible;
use sov_modules_api::test_utils::generate_address as gen_addr;
use sov_modules_api::Address;
use sov_modules_api::{CallResponse, Context, Error, Module, Spec, StateCheckpoint, TxEffect};
use sov_test_utils::storage::new_finalized_storage;
// use sov_prover_storage_manager::new_orphan_storage; // TODO: `sov_prover_storage_manager` is deprecated and removed from the sovereign codebase
use sov_state::ProverStorage;
use sov_test_utils::runtime::genesis::optimistic::HighLevelOptimisticGenesisConfig;
use sov_test_utils::runtime::{assert_tx_reverted_with_reason, TestRunner};
use sov_test_utils::{
    generate_optimistic_runtime, AsUser, MockDaSpec, TestStorageSpec, TestUser, TransactionTestCase,
};
use spicenet_time::constants::SLOT_TIME;
use spicenet_time::TimeConfig;
pub type S = sov_test_utils::TestSpec;
pub type Storage = ProverStorage<TestStorageSpec>;

fn generate_address() -> Address<S> {
    Address::new([0; 32])
}

fn generate_address_sequencer(name: &str) -> <S as Spec>::Address {
    gen_addr::<S>(name)
}

fn generate_address_from_bytes(bytes: &[u8; 32]) -> Address<S>
where
    Address<S>: From<[u8; 32]>,
{
    return Address::<S>::from(*bytes);
}

generate_optimistic_runtime!(TestTimeModuleRuntime <= time: TimeModule<S>);

pub struct TestRoles<S: Spec> {
    pub admin: TestUser<S>,
}

fn setup() -> (
    TestRoles<S>,
    TestRunner<TestTimeModuleRuntime<S, MockDaSpec>, S>,
) {
    let genesis_config =
        HighLevelOptimisticGenesisConfig::generate().add_accounts_with_default_balance(1);

    let time_admin = genesis_config.additional_accounts.first().unwrap().clone();

    let config = TimeConfig::<S> {
        sequencer_authority: generate_address_from_bytes(time_admin.address().as_bytes()),
    };

    let genesis_config =
        GenesisConfig::from_minimal_config(genesis_config.clone().into(), config.clone());

    let runner = TestRunner::new_with_genesis(
        genesis_config.into_genesis_params(),
        TestTimeModuleRuntime::default(),
    );

    (TestRoles { admin: time_admin }, runner)
}

#[test]
fn changeTime() {
    let (TestRoles { admin, .. }, mut runner) = setup();

    let prev_time = runner.query_state(|state| {
        TimeModule::<S>::default()
            .get_time(state)
            .unwrap()
            .unix_timestamp
    });

    assert_ne!(prev_time, 0);

    runner.execute_transaction(TransactionTestCase {
        input: admin.create_plain_message::<TimeModule<S>>(CallMessage::UpdateTimestamp {}),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestTimeModuleRuntimeEvent::Time(Event::UpdateTime {
                    unix_timestamp: prev_time + SLOT_TIME
                })
            );

            let timeResponse = TimeModule::<S>::default().get_time(state).unwrap();

            assert_eq!(timeResponse.unix_timestamp, prev_time + SLOT_TIME);
        }),
    });
}

#[test]
fn changeSlot() {
    let (TestRoles { admin, .. }, mut runner) = setup();

    let prev_slot =
        runner.query_state(|state| TimeModule::<S>::default().get_slot(state).unwrap().slot);

    runner.execute_transaction(TransactionTestCase {
        input: admin.create_plain_message::<TimeModule<S>>(CallMessage::UpdateSlot {}),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestTimeModuleRuntimeEvent::Time(Event::UpdateSlot {
                    slot: prev_slot + 1
                })
            );

            let slotResponse = TimeModule::<S>::default().get_slot(state).unwrap();

            assert_eq!(slotResponse.slot, prev_slot + 1);
        }),
    });
}
