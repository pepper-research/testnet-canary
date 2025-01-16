mod helpers;
mod db;
mod schema;

use bigdecimal::ToPrimitive;
use std::str::FromStr;
use std::{env, thread};
use std::io::Error;
use helpers::read_private_keys;
use sov_modules_api::execution_mode::Native;
use sov_sequencer_json_client::Client as SequencerClient;
use sov_ledger_json_client::Client as LedgerClient;
use clap::Parser;
use jsonrpsee::ws_client::WsClientBuilder;
use reqwest::ClientBuilder;
use sov_mock_da::MockDaSpec;
use sov_mock_zkvm::crypto::private_key::Ed25519PrivateKey;
use sov_modules_api::{CredentialId, CryptoSpec, PrivateKey, PublicKey, Spec};
use spicenet_stf::RuntimeCall;
use sov_modules_api::macros::config_value;
use sov_modules_api::transaction::{PriorityFeeBips, Transaction, UnsignedTransaction};
use sov_modules_api::rest::utils::ResponseObject;
use crate::helpers::publish_batch;

pub const MAX_TX_FEE: u64 = 100_000_000;

pub type MetricSpec = sov_modules_api::default_spec::DefaultSpec<
    sov_mock_da::MockDaSpec,
    sov_mock_zkvm::MockZkVerifier,
    sov_mock_zkvm::MockZkVerifier,
    Native,
>;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    #[arg(long, default_value = "12345")]
    port: String,
    #[arg(long, default_value = "20")]
    max_tps: u32,
    #[arg(long, default_value = "5")]
    epochs: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let max_tps = args.max_tps;
    let epochs = args.epochs;

    for i in 1..(epochs+1) {
        let threads: Vec<_> = (0..((max_tps/epochs)*i))
            .map(|i| {
                thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(
                        async move {
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

                            let mut connection = db::utils::establish_connection();

                            let start = std::time::Instant::now();
                            let tx1 = create_transaction(key_1.clone(), user_address_1.clone(), key_2.clone(), user_address_2.clone()).await;
                            let tx2 = create_transaction(key_2.clone(), user_address_2.clone(), key_1.clone(), user_address_1.clone()).await;
                            let batch_res = publish_batch(sequencer_client.clone(), ledger_client.clone(), &[tx1, tx2]).await;
                            match batch_res {
                                Ok(_) => {
                                    let duration = start.elapsed();
                                    db::spam_res::insert_res(&mut connection, ((max_tps / epochs) * i) as i32, duration.as_millis() as i32, true);
                                    return Ok(());
                                }
                                Err(e) => {
                                    db::spam_res::insert_res(&mut connection, 0, 0, true);
                                    return Err(e);
                                }
                            }
                        }
                    )
                })
            })
            .collect();

        for handle in threads {
            handle.join().unwrap();
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct NonceResponse {
    key: CredentialId,
    value: u64,
}

async fn create_transaction(
    sender: Ed25519PrivateKey,
    sender_address: <MetricSpec as Spec>::Address,
    receiver: Ed25519PrivateKey,
    receiver_address: <MetricSpec as Spec>::Address,
) -> Transaction<MetricSpec> {
    let msg = RuntimeCall::<MetricSpec>::Bank(
        sov_bank::CallMessage::<MetricSpec>::Transfer {
            to: receiver_address,
            coins: sov_bank::Coins {
                amount: 5000,
                token_id: sov_bank::TokenId::from_str(
                    "token_1rwrh8gn2py0dl4vv65twgctmlwck6esm2as9dftumcw89kqqn3nqrduss6",
                )
                    .unwrap(),
            },
        },
    );

    let chain_id = config_value!("CHAIN_ID");
    let credential_id = sender
        .pub_key()
        .credential_id::<<<MetricSpec as Spec>::CryptoSpec as CryptoSpec>::Hasher>();

    let nonce_url = format!(
        "{}/modules/nonces/state/nonces/items/{}",
        "http://127.0.0.1:12345", credential_id
    );

    let http_client = ClientBuilder::default()
        .build()
        .map_err(|e| anyhow::anyhow!(e)).unwrap();

    let response = http_client.get(&nonce_url).send().await.unwrap();
    let response = response.json::<ResponseObject<NonceResponse>>().await.unwrap();

    let nonce = response.data.map(|data| data.value).unwrap_or_default();

    let max_priority_fee = PriorityFeeBips::ZERO;
    let gas_limit = None;

    let tx = Transaction::<MetricSpec>::new_signed_tx(
        &sender,
        UnsignedTransaction::new(
            borsh::to_vec(&msg).unwrap(),
            chain_id,
            max_priority_fee,
            MAX_TX_FEE,
            nonce,
            gas_limit,
        ),
    );

    tx
}

