use crate::Latency;
use anyhow::Context;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use borsh::BorshSerialize;
use futures::StreamExt;
use jsonrpsee::core::__reexports::serde_json;
use sov_cli::wallet_state::PrivateKeyAndAddress;
use sov_ledger_json_client::Client as LedgerClient;
use sov_modules_api::Spec;
use sov_sequencer_json_client::Client as SequencerClient;
use std::path::Path;
use std::time::{Duration, Instant};

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

pub async fn publish_batch<Tx: BorshSerialize>(
    sequencer_client: SequencerClient,
    ledger_client: LedgerClient,
    txs: &[Tx],
    wait_for_processing: bool,
    latency_object: &mut Latency,
) -> anyhow::Result<()> {
    let batch_time = Instant::now();
    let mut raw_txs = vec![];
    for tx in txs {
        let tx_bytes = borsh::to_vec(tx).expect("Borsch error");
        raw_txs.push(tx_bytes)
    }
    let response = sequencer_client
        .publish_batch(&sov_sequencer_json_client::types::PublishBatchBody {
            transactions: raw_txs
                .into_iter()
                .map(|tx| BASE64_STANDARD.encode(tx))
                .collect(),
        })
        .await
        .context("Unable to publish batch")?;

    let response_data = response
        .data
        .as_ref()
        .ok_or(anyhow::anyhow!("No data in response"))?;

    println!(
        "Your batch was submitted to the sequencer for publication. Response: {:?}",
        response_data
    );

    latency_object.set_publish_batch_latency(batch_time.elapsed().as_micros());

    let confirmation_time = Instant::now();

    let target_da_height: u64 = response_data
        .da_height
        .try_into()
        .expect("da_height is out of range");
    let max_waiting_time = Duration::from_secs(300);
    println!(
        "Going to wait for target slot number {} to be processed, up to {:?}",
        target_da_height, max_waiting_time
    );
    let start_wait = Instant::now();

    // Subscribe to slots only to check our batch if the slot has been published.
    let mut slot_subscription = ledger_client.subscribe_slots().await?;

    while start_wait.elapsed() < max_waiting_time {
        if let Some(latest_slot) = slot_subscription.next().await.transpose()? {
            if latest_slot.number >= target_da_height {
                println!(
                    "Rollup has processed target DA height={}!",
                    target_da_height
                );
                latency_object.set_confirmation_latency(confirmation_time.elapsed().as_micros());
                return Ok(());
            }
        }
    }
    anyhow::bail!(
        "Giving up waiting for target batch to be published after {:?}",
        start_wait.elapsed()
    );
}
