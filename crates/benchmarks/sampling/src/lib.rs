mod db;
mod helpers;
mod latency;
mod schema;

use crate::db::samples::insert_sample;
use crate::db::utils::establish_connection;
use crate::latency::{measure_e2e_latency, measure_ping_latency};
use anyhow::Context;
use base64::Engine;
use borsh::BorshSerialize;
use chrono::Utc;
use cron::Schedule;
use futures::StreamExt;
use sov_cli::wallet_state::PrivateKeyAndAddress;
use sov_modules_api::execution_mode::Native;
use sov_modules_api::prelude::serde_json;
use sov_modules_api::{CryptoSpec, PrivateKey, Spec};
use sov_nonces::NoncesRpcClient;
use sov_rollup_interface::crypto::PublicKey;
use std::path::Path;
use std::str::FromStr;
// lazy_static! {
//     static ref RPC_URL: String = std::env::var("RPC_URL").unwrap();
//     static ref RPC_PORT: String = std::env::var("RPC_PORT").unwrap();
// }

pub const MAX_TX_FEE: u64 = 100_000_000;

pub type MetricSpec = sov_modules_api::default_spec::DefaultSpec<
    sov_mock_zkvm::MockZkVerifier,
    sov_mock_zkvm::MockZkVerifier,
    Native,
>;

pub fn read_private_keys<S: Spec>(suffix: &str) -> PrivateKeyAndAddress<S> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let private_keys_dir = Path::new(&manifest_dir).join("../../../test-data/keys");

    let data = std::fs::read_to_string(private_keys_dir.join(suffix))
        .expect("Unable to read file to string");

    let key_and_address: PrivateKeyAndAddress<S> =
        serde_json::from_str(&data).unwrap_or_else(|_| {
            panic!("Unable to convert data {} to PrivateKeyAndAddress", &data);
        });

    assert!(
        key_and_address.is_matching_to_default(),
        "Inconsistent key data"
    );

    key_and_address
}

pub struct Latency {
    pub nonce_latency: Option<u128>,
    pub publish_batch_latency: Option<u128>,
    pub ping_latency: Option<u128>,
    pub confirmation_latency: Option<u128>,
    pub e2e_latency: Option<u128>,
}

impl Latency {
    pub fn new() -> Self {
        Latency {
            nonce_latency: None,
            publish_batch_latency: None,
            ping_latency: None,
            confirmation_latency: None,
            e2e_latency: None,
        }
    }

    pub fn set_nonce_latency(&mut self, latency: u128) {
        self.nonce_latency = Some(latency);
    }

    pub fn set_publish_batch_latency(&mut self, latency: u128) {
        self.publish_batch_latency = Some(latency);
    }

    pub fn set_ping_latency(&mut self, latency: u128) {
        self.ping_latency = Some(latency);
    }

    pub fn set_confirmation_latency(&mut self, latency: u128) {
        self.confirmation_latency = Some(latency);
    }

    pub fn set_e2e_latency(&mut self, latency: u128) {
        self.e2e_latency = Some(latency);
    }

    pub fn verify(&self) -> bool {
        self.nonce_latency.is_some()
            && self.publish_batch_latency.is_some()
            && self.ping_latency.is_some()
            && self.confirmation_latency.is_some()
            && self.e2e_latency.is_some()
    }
}

#[tokio::main]
async fn main() {
    let mut connection = establish_connection();
    let expression = "0 */3 * * * * *";
    let schedule = Schedule::from_str(expression).unwrap();

    while let Some(time) = schedule.upcoming(Utc).next() {
        let now = chrono::Utc::now();
        if time > now {
            let duration = time - now;
            tokio::time::sleep(tokio::time::Duration::from_secs(
                duration.num_seconds() as u64
            ))
            .await;
        };
        println!("Running Sampler");
        let mut latency_object = Latency::new();

        measure_ping_latency(&mut latency_object);

        measure_e2e_latency(&mut latency_object).await;

        if latency_object.verify() {
            println!("Object verified");
            insert_sample(&mut connection, latency_object);
        };
    }
}
