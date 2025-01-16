use crate::helpers::{publish_batch, read_private_keys};
use crate::{Latency, MetricSpec, MAX_TX_FEE};
use dns_lookup::lookup_host;
use fastping_rs::PingResult::{Idle, Receive};
use fastping_rs::Pinger;
use jsonrpsee::ws_client::WsClientBuilder;
use sov_bank::TokenId;
use sov_ledger_json_client::Client as LedgerClient;
use sov_mock_da::MockDaSpec;
use sov_modules_api::macros::config_value;
use sov_modules_api::transaction::{PriorityFeeBips, Transaction, UnsignedTransaction};
use sov_modules_api::{CryptoSpec, PrivateKey, Spec};
use sov_nonces::NoncesRpcClient;
use sov_rollup_interface::crypto::PublicKey;
use sov_sequencer_json_client::Client as SequencerClient;
use std::str::FromStr;
use std::time::Instant;
use spicenet_stf::RuntimeCall;

pub async fn measure_e2e_latency(latency_object: &mut Latency) {
    let key_and_address_1 = read_private_keys::<MetricSpec>("sampler1.json");
    let key_1 = key_and_address_1.private_key;
    let user_address_1: <MetricSpec as Spec>::Address = key_and_address_1.address;

    let key_and_address_2 = read_private_keys::<MetricSpec>("sampler2.json");
    let key_2 = key_and_address_2.private_key;
    let user_address_2: <MetricSpec as Spec>::Address = key_and_address_2.address;

    let ledger_url = format!("http://127.0.0.1:12346/ledger");
    let ledger_client = LedgerClient::new(&ledger_url);

    let sequencer_url = format!("http://127.0.0.1:12346/sequencer");
    let sequencer_client = SequencerClient::new(&sequencer_url);

    let msg = RuntimeCall::<MetricSpec, MockDaSpec>::Bank(
        sov_bank::CallMessage::<MetricSpec>::Transfer {
            to: user_address_2,
            coins: sov_bank::Coins {
                amount: 5000,
                token_id: TokenId::from_str(
                    "token_1rwrh8gn2py0dl4vv65twgctmlwck6esm2as9dftumcw89kqqn3nqrduss6",
                )
                .unwrap(),
            },
        },
    );

    let chain_id = config_value!("CHAIN_ID");
    let credential_id = key_1
        .pub_key()
        .credential_id::<<<MetricSpec as Spec>::CryptoSpec as CryptoSpec>::Hasher>();

    let rpc = WsClientBuilder::default()
        .build(&format!("ws://127.0.0.1:12345"))
        .await
        .expect("WS Errror");

    let nonces_fetch_time = Instant::now();
    let nonce = NoncesRpcClient::<MetricSpec>::get_nonce(&rpc, credential_id)
        .await
        .expect("Nonce error")
        .nonce;
    latency_object.set_nonce_latency(nonces_fetch_time.elapsed().as_micros());
    let max_priority_fee = PriorityFeeBips::ZERO;
    let gas_limit = None;

    let tx = Transaction::<MetricSpec>::new_signed_tx(
        &key_1,
        UnsignedTransaction::new(
            borsh::to_vec(&msg).unwrap(),
            chain_id,
            max_priority_fee,
            MAX_TX_FEE,
            nonce,
            gas_limit,
        ),
    );

    let e2e_time = Instant::now();
    publish_batch(sequencer_client, ledger_client, &[tx], true, latency_object)
        .await
        .expect("Publish Batch error");
    latency_object.set_e2e_latency(e2e_time.elapsed().as_micros());
    revert_funds();
}

async fn revert_funds() {
    let key_and_address_1 = read_private_keys::<MetricSpec>("sampler1.json");
    let key_1 = key_and_address_1.private_key;
    let user_address_1: <MetricSpec as Spec>::Address = key_and_address_1.address;

    let key_and_address_2 = read_private_keys::<MetricSpec>("sampler2.json");
    let key_2 = key_and_address_2.private_key;
    let user_address_2: <MetricSpec as Spec>::Address = key_and_address_2.address;

    let ledger_url = format!("http://127.0.0.1:12346/ledger");
    let ledger_client = LedgerClient::new(&ledger_url);

    let sequencer_url = format!("http://127.0.0.1:12346/sequencer");
    let sequencer_client = SequencerClient::new(&sequencer_url);

    let msg = RuntimeCall::<MetricSpec, MockDaSpec>::Bank(
        sov_bank::CallMessage::<MetricSpec>::Transfer {
            to: user_address_1,
            coins: sov_bank::Coins {
                amount: 5000,
                token_id: TokenId::from_str(
                    "token_1rwrh8gn2py0dl4vv65twgctmlwck6esm2as9dftumcw89kqqn3nqrduss6",
                )
                .unwrap(),
            },
        },
    );

    let chain_id = config_value!("CHAIN_ID");
    let credential_id = key_2
        .pub_key()
        .credential_id::<<<MetricSpec as Spec>::CryptoSpec as CryptoSpec>::Hasher>();

    let rpc = WsClientBuilder::default()
        .build(&format!("ws://127.0.0.1:12345"))
        .await
        .expect("WS Errror");

    let nonce = NoncesRpcClient::<MetricSpec>::get_nonce(&rpc, credential_id)
        .await
        .expect("Nonce error")
        .nonce;
    let max_priority_fee = PriorityFeeBips::ZERO;
    let gas_limit = None;

    let tx = Transaction::<MetricSpec>::new_signed_tx(
        &key_2,
        UnsignedTransaction::new(
            borsh::to_vec(&msg).unwrap(),
            chain_id,
            max_priority_fee,
            MAX_TX_FEE,
            nonce,
            gas_limit,
        ),
    );

    sequencer_client
        .publish_batch_with_serialized_txs(&[tx])
        .await
        .unwrap();
}

pub fn measure_ping_latency(latency_object: &mut Latency) {
    let rpc_ip = lookup_host("localhost").unwrap();
    let (pinger, results) = match Pinger::new(None, Some(56)) {
        Ok((pinger, results)) => (pinger, results),
        Err(e) => panic!("Error creating pinger: {}", e),
    };
    pinger.add_ipaddr(&rpc_ip.first().unwrap().to_string());
    pinger.run_pinger();
    let mut sum_ping_times: u128 = 0;
    let mut pings: u128 = 0;
    for i in 0..10 {
        match results.recv() {
            Ok(result) => match result {
                Idle { addr } => {}
                Receive { addr, rtt } => {
                    sum_ping_times += rtt.as_micros();
                    pings += 1;
                }
            },
            Err(_) => panic!("Worker threads disconnected before the solution was found!"),
        }
    }

    pinger.stop_pinger();

    latency_object.set_ping_latency(sum_ping_times / pings);
}
