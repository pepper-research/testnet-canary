#![allow(dead_code)]
use std::convert::Infallible;

use spicenet_risk::{call::CallMessage, event::Event, RiskModule};

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
use spicenet_risk::genesis::RiskModuleConfig;
use spicenet_time::{TimeConfig, TimeModule};

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

generate_optimistic_runtime!(TestRuntime <= registry: RiskModule<S>, time: TimeModule<S>, lut: lut::LookupTable<S>);

pub struct TestRoles<S: Spec> {
    pub admin: TestUser<S>,
    pub wallet1: TestUser<S>,
    pub wallet2: TestUser<S>,
}

fn setup() -> (TestRoles<S>, TestRunner<TestRuntime<S, MockDaSpec>, S>) {
    let genesis_config =
        HighLevelOptimisticGenesisConfig::generate().add_accounts_with_default_balance(3);

    let admin = genesis_config.additional_accounts.first().unwrap().clone();

    let wallet1 = genesis_config.additional_accounts[1].clone();

    let wallet2 = genesis_config.additional_accounts[2].clone();

    let riskConfig = RiskModuleConfig {};

    let timeConfig = TimeConfig::<S> {
        sequencer_authority: generate_address_from_bytes(admin.address().as_bytes()),
    };

    let lutConfig = lut::LookupTableConfig {
        prices: [34],
        aggregate_conf_intervals: [12],
    };

    let genesis_config = GenesisConfig::from_minimal_config(
        genesis_config.clone().into(),
        riskConfig.clone(),
        timeConfig.clone(),
        lutConfig.clone(),
    );

    let runner =
        TestRunner::new_with_genesis(genesis_config.into_genesis_params(), TestRuntime::default());

    (
        TestRoles {
            admin,
            wallet1,
            wallet2,
        },
        runner,
    )
}

#[test]
fn checkBankBalances() {
    let (
        TestRoles {
            admin,
            wallet1,
            wallet2,
        },
        mut runner,
    ) = setup();
}
