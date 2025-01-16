#![allow(dead_code)]
use std::convert::Infallible;

use oracle_registry::{call::CallMessage, event::Event, OracleRegistry};

use sov_modules_api::prelude::UnwrapInfallible;
use sov_modules_api::test_utils::generate_address as gen_addr;
use sov_modules_api::Address;
use sov_modules_api::{CallResponse, Context, Error, Module, Spec, StateCheckpoint, TxEffect};
use sov_test_utils::storage::new_finalized_storage;
// use sov_prover_storage_manager::new_orphan_storage; // TODO: `sov_prover_storage_manager` is deprecated and removed from the sovereign codebase
use oracle_registry::OracleRegistryConfig;
use sov_bank::{Bank, GAS_TOKEN_ID};
use sov_state::ProverStorage;
use sov_test_utils::runtime::genesis::optimistic::HighLevelOptimisticGenesisConfig;
use sov_test_utils::runtime::{assert_tx_reverted_with_reason, TestRunner};
use sov_test_utils::{
    generate_optimistic_runtime, AsUser, MockDaSpec, TestStorageSpec, TestUser, TransactionTestCase,
};
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

generate_optimistic_runtime!(TestRuntime <= registry: OracleRegistry<S>, time: TimeModule<S>);

pub struct TestRoles<S: Spec> {
    pub admin: TestUser<S>,
    pub node: TestUser<S>,
    pub user: TestUser<S>,
}

fn setup() -> (TestRoles<S>, TestRunner<TestRuntime<S, MockDaSpec>, S>) {
    let genesis_config =
        HighLevelOptimisticGenesisConfig::generate().add_accounts_with_default_balance(3);

    let registry_admin = genesis_config.additional_accounts.first().unwrap().clone();

    let registry_user = genesis_config.additional_accounts[1].clone();

    let registry_node = genesis_config.additional_accounts[2].clone();

    let registryConfig = OracleRegistryConfig::<S> {
        registry_authority: generate_address_from_bytes(registry_admin.address().as_bytes()),
        minimum_bond_amt: 10u64,
    };

    let timeConfig = TimeConfig::<S> {
        sequencer_authority: generate_address_from_bytes(registry_admin.address().as_bytes()),
    };

    let genesis_config = GenesisConfig::from_minimal_config(
        genesis_config.clone().into(),
        registryConfig.clone(),
        timeConfig.clone(),
    );

    let runner =
        TestRunner::new_with_genesis(genesis_config.into_genesis_params(), TestRuntime::default());

    (
        TestRoles {
            admin: registry_admin,
            node: registry_node,
            user: registry_user,
        },
        runner,
    )
}

#[test]
fn checkBankBalances() {
    let (TestRoles { admin, node, user }, mut runner) = setup();

    let admin_balance = runner.query_state(|state| {
        Bank::<S>::default()
            .get_balance_of(&admin.address(), GAS_TOKEN_ID, state)
            .unwrap()
            .unwrap()
    });

    assert_eq!(admin_balance, 1000000000);

    let node_balance = runner.query_state(|state| {
        Bank::<S>::default()
            .get_balance_of(&node.address(), GAS_TOKEN_ID, state)
            .unwrap()
            .unwrap()
    });

    assert_eq!(node_balance, 1000000000);

    let user_balance = runner.query_state(|state| {
        Bank::<S>::default()
            .get_balance_of(&user.address(), GAS_TOKEN_ID, state)
            .unwrap()
            .unwrap()
    });

    assert_eq!(user_balance, 1000000000);
}

#[test]
fn fail_unauthorized_whitelist() {
    let (TestRoles { admin, node, user }, mut runner) = setup();

    let tx = TransactionTestCase {
        input: node.create_plain_message::<OracleRegistry<S>>(CallMessage::Whitelist {
            user_address: generate_address_from_bytes(user.address().as_bytes()),
        }),
        assert: Box::new(move |result, _state| {
            assert_tx_reverted_with_reason(
                result.tx_receipt,
                anyhow::anyhow!("Sender is not the registry authority"),
            );
        }),
    };

    runner.execute_transaction(tx);
}

#[test]
pub fn whitelist_user() {
    let (TestRoles { admin, node, user }, mut runner) = setup();

    let tx = TransactionTestCase {
        input: admin.create_plain_message::<OracleRegistry<S>>(CallMessage::Whitelist {
            user_address: generate_address_from_bytes(user.address().as_bytes()),
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestRuntimeEvent::Registry(Event::UserWhitelisted {
                    user_address: generate_address_from_bytes(user.address().as_bytes())
                })
            );

            let whitelisted_user = OracleRegistry::<S>::default()
                .get_whitelisted_user(
                    state,
                    generate_address_from_bytes(user.address().as_bytes()),
                )
                .unwrap();

            assert_eq!(
                whitelisted_user.address.unwrap(),
                generate_address_from_bytes(user.address().as_bytes())
            );
            assert_eq!(
                whitelisted_user.whitelisted_ts.unwrap(),
                TimeModule::<S>::default()
                    .get_time(state)
                    .unwrap()
                    .unix_timestamp
            );
            assert_eq!(whitelisted_user.is_oracle_node.unwrap(), false);
        }),
    };

    runner.execute_transaction(tx);
}

#[test]
fn fail_insufficient_funds() {
    let (TestRoles { admin, node, user }, mut runner) = setup();

    // whitelist
    runner.execute(
        admin.create_plain_message::<OracleRegistry<S>>(CallMessage::Whitelist {
            user_address: generate_address_from_bytes(user.address().as_bytes()),
        }),
    );

    let tx = TransactionTestCase {
        input: user.create_plain_message::<OracleRegistry<S>>(CallMessage::Register {
            node_address: generate_address_from_bytes(node.address().as_bytes()),
            user_address: generate_address_from_bytes(user.address().as_bytes()),
            amount: 1000000001,
        }),
        assert: Box::new(move |result, _state| assert!(result.tx_receipt.is_reverted())),
    };

    runner.execute_transaction(tx);
}

#[test]
fn register_node() {
    let (TestRoles { admin, node, user }, mut runner) = setup();

    // whitelist
    runner.execute(
        admin.create_plain_message::<OracleRegistry<S>>(CallMessage::Whitelist {
            user_address: generate_address_from_bytes(user.address().as_bytes()),
        }),
    );

    let tx = TransactionTestCase {
        input: user.create_plain_message::<OracleRegistry<S>>(CallMessage::Register {
            node_address: generate_address_from_bytes(node.address().as_bytes()),
            user_address: generate_address_from_bytes(user.address().as_bytes()),
            amount: 100,
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestRuntimeEvent::Registry(Event::NodeRegistered {
                    node_address: generate_address_from_bytes(node.address().as_bytes()),
                    amount: 100
                })
            );

            let oracle_node = OracleRegistry::<S>::default()
                .get_oracle_node(
                    state,
                    generate_address_from_bytes(node.address().as_bytes()),
                )
                .unwrap();

            assert_eq!(
                oracle_node.address.unwrap(),
                generate_address_from_bytes(node.address().as_bytes())
            );
            assert_eq!(oracle_node.amount_staked.unwrap(), 100);
            assert_eq!(oracle_node.accumulated_penalty.unwrap(), 0);
        }),
    };

    runner.execute_transaction(tx);
}

#[test]
fn deposit_more() {
    let (TestRoles { admin, node, user }, mut runner) = setup();

    // whitelist
    runner.execute(
        admin.create_plain_message::<OracleRegistry<S>>(CallMessage::Whitelist {
            user_address: generate_address_from_bytes(user.address().as_bytes()),
        }),
    );

    // register
    runner.execute(
        user.create_plain_message::<OracleRegistry<S>>(CallMessage::Register {
            node_address: generate_address_from_bytes(node.address().as_bytes()),
            user_address: generate_address_from_bytes(user.address().as_bytes()),
            amount: 100,
        }),
    );

    let tx = TransactionTestCase {
        input: user.create_plain_message::<OracleRegistry<S>>(CallMessage::Deposit {
            node_address: generate_address_from_bytes(node.address().as_bytes()),
            user_address: generate_address_from_bytes(user.address().as_bytes()),
            amount: 100,
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestRuntimeEvent::Registry(Event::NodeDeposited {
                    node_address: generate_address_from_bytes(node.address().as_bytes()),
                    amount: 100
                })
            );

            let oracle_node = OracleRegistry::<S>::default()
                .get_oracle_node(
                    state,
                    generate_address_from_bytes(node.address().as_bytes()),
                )
                .unwrap();

            assert_eq!(
                oracle_node.address.unwrap(),
                generate_address_from_bytes(node.address().as_bytes())
            );
            assert_eq!(oracle_node.amount_staked.unwrap(), 200);
            assert_eq!(oracle_node.accumulated_penalty.unwrap(), 0);
        }),
    };

    runner.execute_transaction(tx);
}

#[test]
fn exit() {
    let (TestRoles { admin, node, user }, mut runner) = setup();

    // whitelist
    runner.execute(
        admin.create_plain_message::<OracleRegistry<S>>(CallMessage::Whitelist {
            user_address: generate_address_from_bytes(user.address().as_bytes()),
        }),
    );

    // register
    runner.execute(
        user.create_plain_message::<OracleRegistry<S>>(CallMessage::Register {
            node_address: generate_address_from_bytes(node.address().as_bytes()),
            user_address: generate_address_from_bytes(user.address().as_bytes()),
            amount: 100,
        }),
    );

    let tx = TransactionTestCase {
        input: user.create_plain_message::<OracleRegistry<S>>(CallMessage::Exit {
            node_address: generate_address_from_bytes(node.address().as_bytes()),
            user_address: generate_address_from_bytes(user.address().as_bytes()),
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestRuntimeEvent::Registry(Event::NodeExited {
                    node_address: generate_address_from_bytes(node.address().as_bytes())
                })
            );

            let oracle_node = OracleRegistry::<S>::default()
                .get_oracle_node(
                    state,
                    generate_address_from_bytes(node.address().as_bytes()),
                )
                .unwrap();

            // assert!(oracle_node.unwrap());
            // println!("{:?}",oracle_node.unwrap())

            assert!(oracle_node.address.is_none());
            assert!(oracle_node.amount_staked.is_none());
            assert!(oracle_node.accumulated_penalty.is_none());
        }),
    };

    runner.execute_transaction(tx);
}
