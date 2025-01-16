#![allow(dead_code)]
use std::convert::Infallible;

use lut::{AggrDataResponse, CallMessage, Event, LookupTable};

use sov_modules_api::prelude::UnwrapInfallible;
use sov_modules_api::test_utils::generate_address as gen_addr;
use sov_modules_api::{
    Address, CallResponse, Context, Error, Module, Spec, StateCheckpoint, TxEffect,
};
use sov_test_utils::storage::new_finalized_storage;
// use sov_prover_storage_manager::new_orphan_storage; // TODO: `sov_prover_storage_manager` is deprecated and removed from the sovereign codebase
use lut::LookupTableConfig;
use sov_state::ProverStorage;
use sov_test_utils::runtime::genesis::optimistic::HighLevelOptimisticGenesisConfig;
use sov_test_utils::runtime::{assert_tx_reverted_with_reason, TestRunner};
use sov_test_utils::{
    generate_optimistic_runtime, AsUser, MockDaSpec, TestStorageSpec, TestUser, TransactionTestCase,
};
use spicenet_time::{TimeConfig, TimeModule};
pub type S = sov_test_utils::TestSpec;
pub type Storage = ProverStorage<TestStorageSpec>;
// wrapper for sov generate_address with generic S
fn generate_address(name: &str) -> <S as Spec>::Address {
    gen_addr::<S>(name)
}

generate_optimistic_runtime!(TestLUTModuleRuntime <= lut: LookupTable<S>, time: TimeModule<S>);

pub struct TestRoles<S: Spec> {
    pub admin: TestUser<S>,
}

fn generate_address_from_bytes(bytes: &[u8; 32]) -> Address<S>
where
    Address<S>: From<[u8; 32]>,
{
    return Address::<S>::from(*bytes);
}

fn setup() -> (
    TestRoles<S>,
    TestRunner<TestLUTModuleRuntime<S, MockDaSpec>, S>,
) {
    let genesis_config =
        HighLevelOptimisticGenesisConfig::generate().add_accounts_with_default_balance(1);

    let lut_admin = genesis_config.additional_accounts.first().unwrap().clone();

    let lut_config = LookupTableConfig {
        prices: [1u64],
        aggregate_conf_intervals: [0u32],
        update_authority: generate_address_from_bytes(lut_admin.address().as_bytes()),
    };

    let time_config = TimeConfig {
        sequencer_authority: generate_address_from_bytes(lut_admin.address().as_bytes()),
    };

    let genesis_config = GenesisConfig::from_minimal_config(
        genesis_config.clone().into(),
        lut_config.clone(),
        time_config.clone(),
    );

    let runner = TestRunner::new_with_genesis(
        genesis_config.into_genesis_params(),
        TestLUTModuleRuntime::default(),
    );

    (TestRoles { admin: lut_admin }, runner)
}

#[test]
fn changePrices() {
    let (TestRoles { admin, .. }, mut runner) = setup();

    runner.execute_transaction(TransactionTestCase {
        input: admin.create_plain_message::<LookupTable<S>>(CallMessage::MutateAll {
            prices: [34.into()],
            aggregate_conf_intervals: [12],
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestLutModuleRuntimeEvent::Lut(Event::MutateAll {
                    prices: [34.into()],
                    aggregate_conf_intervals: [12]
                })
            );

            let pricesResponse = LookupTable::<S>::default().get_all_prices(state).unwrap();

            assert_eq!(pricesResponse.prices, [34.into()]);

            let aggrDataResponse = LookupTable::<S>::default()
                .get_all_aggr_conf_intervals(state)
                .unwrap();

            assert_eq!(aggrDataResponse.aggregate_conf_intervals, [12]);
        }),
    });
}

// #[test]
// fn genesis_prices() -> Result<(), Infallible> {
//     // let admin = generate_address("admin");
//     let sequencer = generate_address("sequencer");
//     // let sequencer = generate_address("sequencer");
//     println!("config is working");
//     let temp_dir = tempfile::tempdir().unwrap();
//     let state = StateCheckpoint::<S>::new(new_finalized_storage(temp_dir.path()));
//     let mut lut = LookupTable::default();

// let config = LookupTableConfig {
//     prices: [1u64],
//     aggregate_conf_intervals: [0u32]
// };

//     println!("config:{:?}", config);

//     // TODO: figure out `state.to_genesis_state_accessor` function
//     let mut genesis_state = state.to_genesis_state_accessor::<LookupTable<S>>(&config);

//     // genesis
//     let genesis_result = lut.genesis(&config, &mut genesis_state);
//     assert!(genesis_result.is_ok());

//     println!("{:?}", lut.get_all_prices(&mut genesis_state).unwrap().prices);

//     let mut checkpoint = genesis_state.checkpoint();
//     let mut state = checkpoint.to_working_set_unmetered();

//     let mutate_all_message = CallMessage::MutateAll {
//         prices: [34],
//         aggregate_conf_intervals: [12]
//     };
//     let temp_context = Context::<S>::new(sequencer, Default::default(), sequencer, 1);
//     lut.call(mutate_all_message.clone(), &temp_context, &mut state).expect("TODO: panic message");

//     assert_eq!(vec![34], lut.get_all_prices(&mut state).unwrap().prices);
//     assert_eq!(vec![12], lut.get_all_aggr_conf_intervals(&mut state).unwrap().aggregate_conf_intervals);

//     let typed_event = state.take_event(0).unwrap();

//     assert_eq!(
//         typed_event.downcast::<Event>().unwrap(),
//         Event::MutateAll {
//             prices: [34],
//             aggregate_conf_intervals: [12]
//         }
//     );

//     // #[cfg(feature = "native")]
//     // {
//     //     let mut working_set = state.to_working_set_unmetered();
//     //     let update_prices = lut.update_all([345], [123], &mut working_set);
//     //
//     //     state = working_set.checkpoint().0;
//     //     let all_prices = lut.get_all_prices(&mut state);
//     //
//     //     assert_eq!(true, update_prices.unwrap().ok);
//     //     assert_eq!(vec![34], all_prices.unwrap().prices);
//     // }
//     Ok(())
// }
