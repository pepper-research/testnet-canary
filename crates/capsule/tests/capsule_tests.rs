#![allow(dead_code)]
use std::convert::Infallible;

use capsule::{call::CallMessage, event::Event, Capsule};
use ed25519_dalek::Signature;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::signers::Signers;
use sov_modules_api::prelude::UnwrapInfallible;
use sov_modules_api::test_utils::generate_address as gen_addr;
use sov_modules_api::Address;
use sov_modules_api::{CallResponse, Context, Error, Module, Spec, StateCheckpoint, TxEffect};
use sov_test_utils::storage::new_finalized_storage;
// use sov_prover_storage_manager::new_orphan_storage; // TODO: `sov_prover_storage_manager` is deprecated and removed from the sovereign codebase
use capsule::state::wallet::{Wallet, WalletState, WalletType};
use capsule::CapsuleConfig;
use sov_state::ProverStorage;
use sov_test_utils::runtime::genesis::optimistic::HighLevelOptimisticGenesisConfig;
use sov_test_utils::runtime::{assert_tx_reverted_with_reason, TestRunner};
use sov_test_utils::{
    generate_optimistic_runtime, AsUser, MockDaSpec, TestStorageSpec, TestUser, TransactionTestCase,
};

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

generate_optimistic_runtime!(TestCapsuleModuleRuntime <= capsule: Capsule<S>);

pub struct TestRoles<S: Spec> {
    pub admin: TestUser<S>,
    pub wallet1: TestUser<S>,
}

fn setup() -> (
    TestRoles<S>,
    TestRunner<TestCapsuleModuleRuntime<S, MockDaSpec>, S>,
) {
    let genesis_config =
        HighLevelOptimisticGenesisConfig::generate().add_accounts_with_default_balance(2);

    let time_admin = genesis_config.additional_accounts.first().unwrap().clone();

    let wallet1 = genesis_config.additional_accounts[1].clone();

    let config = CapsuleConfig {};

    let genesis_config =
        GenesisConfig::from_minimal_config(genesis_config.clone().into(), config.clone());

    let runner = TestRunner::new_with_genesis(
        genesis_config.into_genesis_params(),
        TestCapsuleModuleRuntime::default(),
    );

    (
        TestRoles {
            admin: time_admin,
            wallet1: wallet1,
        },
        runner,
    )
}

#[test]
fn addWalletTest() {
    let (TestRoles { admin, wallet1 }, mut runner) = setup();

    let wallet_Type = WalletType::Solana {
        address: *wallet1.address().as_bytes(),
    };
    let nonce = 0;

    let message = format!(
        "I am creating a new smart wallet and adding an admin wallet {wallet_Type}. Nonce: {nonce}"
    );
    let wallet1_kp = Keypair::from_bytes(wallet1.address().as_bytes());
    let binding = wallet1_kp.sign_message(message.as_bytes());
    let signature = binding.first().unwrap();

    runner.execute_transaction(TransactionTestCase {
        input: admin.create_plain_message::<Capsule<S>>(CallMessage::CreateWallet {
            wallet_type: WalletType::Solana {
                address: *wallet1.address().as_bytes(),
            },
            signature: Vec::from(signature.to_string().as_bytes()),
            nonce: 0,
        }),
        assert: Box::new(move |result, state| {
            println!("{:?}", result.tx_receipt);
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);

            let wallet = Capsule::<S>::default()
                .get_wallet(
                    generate_address_from_bytes(wallet1.address().as_bytes()),
                    state,
                )
                .unwrap();

            println!("{:?}", wallet.1);

            // assert_eq!(wallet, Wallet { address: wallet1.address });
        }),
    });
}

// #[test]
// fn changeTime() {
//     let (TestRoles { admin, .. }, mut runner) = setup();
//
//     let prev_time = runner.query_state(|state| {
//        TimeModule::<S>::default().get_time(state).unwrap().unix_timestamp
//     });
//
//     assert_ne!(prev_time,0);
//
//     runner.execute_transaction(TransactionTestCase {
//         input: admin.create_plain_message::<TimeModule<S>>(CallMessage::UpdateTimestamp {}),
//         assert: Box::new(move |result, state| {
//             assert!(result.tx_receipt.is_successful());
//             assert_eq!(result.events.len(), 1);
//             assert_eq!(
//                 result.events[0],
//                 TestTimeModuleRuntimeEvent::Time(Event::UpdateTime {
//                     unix_timestamp: prev_time + SLOT_TIME
//                 })
//             );
//
//             let timeResponse = TimeModule::<S>::default()
//                 .get_time(state)
//                 .unwrap();
//
//             assert_eq!(timeResponse.unix_timestamp, prev_time + SLOT_TIME);
//         })
//     });
//
// }
//
// #[test]
// fn changeSlot() {
//     let (TestRoles { admin, .. }, mut runner) = setup();
//
//     let prev_slot = runner.query_state(|state| {
//        TimeModule::<S>::default().get_slot(state).unwrap().slot
//     });
//
//     runner.execute_transaction(TransactionTestCase {
//         input: admin.create_plain_message::<TimeModule<S>>(CallMessage::UpdateSlot {}),
//         assert: Box::new(move |result, state| {
//             assert!(result.tx_receipt.is_successful());
//             assert_eq!(result.events.len(), 1);
//             assert_eq!(
//                 result.events[0],
//                 TestTimeModuleRuntimeEvent::Time(Event::UpdateSlot {
//                     slot: prev_slot + 1
//                 })
//             );
//
//             let slotResponse = TimeModule::<S>::default()
//                 .get_slot(state)
//                 .unwrap();
//
//             assert_eq!(slotResponse.slot, prev_slot + 1);
//         })
//     });
// }
