#![allow(dead_code)]
use spicenet_aaob::{get_market_id, CallMessage, Event, Market, SelfTradeHandler, AAOB};
use std::convert::Infallible;

use sov_modules_api::prelude::UnwrapInfallible;
use sov_modules_api::test_utils::generate_address as gen_addr;
use sov_modules_api::transaction::{Transaction, UnsignedTransaction};
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

use spicenet_aaob::AAOBConfig;
use spicenet_shared::Side;

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

generate_optimistic_runtime!(TestRuntime <= aaob: AAOB<S>);

pub struct TestRoles<S: Spec> {
    pub admin: TestUser<S>,
    pub user1: TestUser<S>,
    pub user2: TestUser<S>,
}

fn setup() -> (TestRoles<S>, TestRunner<TestRuntime<S, MockDaSpec>, S>) {
    let genesis_config =
        HighLevelOptimisticGenesisConfig::generate().add_accounts_with_default_balance(3);

    let registry_admin = genesis_config.additional_accounts.first().unwrap().clone();

    let user1 = genesis_config.additional_accounts[1].clone();

    let user2 = genesis_config.additional_accounts[2].clone();

    let aaobConfig = AAOBConfig::<S> {
        initial_markets: vec![
            Market {
                name: "TBD/USD".to_string(),
                orderbook_id: 1,
                fee_budget: 10000000,
                min_base_size: 0,
                tick_size: 1000,
                _phantom: std::marker::PhantomData,
            },
            Market {
                name: "AAOB/USD".to_string(),
                orderbook_id: 1,
                fee_budget: 10000000,
                min_base_size: 0,
                tick_size: 1000,
                _phantom: std::marker::PhantomData,
            },
        ],
        _phantom: std::marker::PhantomData,
    };

    let genesis_config =
        GenesisConfig::from_minimal_config(genesis_config.clone().into(), aaobConfig.clone());

    let runner =
        TestRunner::new_with_genesis(genesis_config.into_genesis_params(), TestRuntime::default());

    (
        TestRoles {
            admin: registry_admin,
            user1: user1,
            user2: user2,
        },
        runner,
    )
}

#[test]
fn add_market() {
    let (
        TestRoles {
            admin,
            user1,
            user2,
        },
        mut runner,
    ) = setup();

    let tx = TransactionTestCase {
        input: admin.create_plain_message::<AAOB<S>>(CallMessage::CreateMarket {
            market_name: "PEP/USD".to_string(),
            orderbook_id: 1,
            fee_budget: 10000000,
            min_base_size: 0,
            tick_size: 1000,
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);

            let markets = AAOB::<S>::default().get_markets(state).unwrap();

            println!("{:?}", markets);
        }),
    };
    runner.execute_transaction(tx);
}

#[test]
fn close_market() {
    let (
        TestRoles {
            admin,
            user1,
            user2,
        },
        mut runner,
    ) = setup();

    let tx = TransactionTestCase {
        input: admin.create_plain_message::<AAOB<S>>(CallMessage::CloseMarket {
            market_name: "TBD/USD".to_string(),
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            let markets = AAOB::<S>::default().get_markets(state).unwrap();

            println!("{:?}", markets);

            let orderbook = AAOB::<S>::default().get_orderbook(1, state).unwrap();

            println!("{:?}", orderbook);
        }),
    };
    runner.execute_transaction(tx);
}

#[test]
fn create_order() {
    let (
        TestRoles {
            admin,
            user1,
            user2,
        },
        mut runner,
    ) = setup();

    let market_id = get_market_id::<S>("AAOB/USD");

    let tx = TransactionTestCase {
        input: user1.create_plain_message::<AAOB<S>>(CallMessage::CreateOrder {
            market_id: market_id,
            orderbook_id: 1,
            side: Side::Bid,
            max_base_qty: 0,
            max_quote_qty: 0,
            limit_price: 0,
            post_only: false,
            post_allowed: false,
            self_trade_behavior: SelfTradeHandler::DecrementTake,
            match_limit: 0,
            trg_id: 0,
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
        }),
    };

    runner.execute_transaction(tx);
}
