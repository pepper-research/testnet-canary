use std::path::PathBuf;
use std::process::exit;

use anyhow::Context as _;
use clap::Parser;
use spicenet_stf::genesis_config::GenesisPaths;
use sha2::Sha256;
use sov_celestia_adapter::CelestiaService;
use spicenet_rollup::{celestia_rollup::CelestiaRollup, mock_rollup::MockRollup};
use sov_mock_da::storable::service::StorableMockDaService;
use sov_modules_api::execution_mode::Native;
use sov_modules_api::Address;
use sov_modules_rollup_blueprint::logging::initialize_logging;
use sov_modules_rollup_blueprint::{FullNodeBlueprint, Rollup};
// use sov_rollup_interface::node::da::DaServiceWithRetries;
use sov_stf_runner::processes::RollupProverConfig;
use sov_stf_runner::{from_toml_path, RollupConfig};
use tracing::debug;

/// Main demo runner. Initializes a DA chain, and starts a demo-rollup using the provided.
/// If you're trying to sign or submit transactions to the rollup, the `sov-cli` binary
/// is the one you want. You can run it `cargo run --bin sov-cli`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The data layer type.
    #[arg(long, default_value = "mock")]
    da_layer: SupportedDaLayer,

    /// The path to the rollup config.
    #[arg(long, default_value = "../../rollup_config.toml")]
    rollup_config_path: String,

    /// The path to the genesis configs.
    #[arg(long, default_value = "../../test-data/genesis/mock")]
    genesis_config_dir: PathBuf,

    /// Listen address for Prometheus exporter.
    #[arg(long, default_value = "127.0.0.1:9845")]
    prometheus_exporter_bind: String,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum SupportedDaLayer {
    Celestia,
    Mock,
}

#[tokio::main]
async fn main() {
    initialize_logging();

    match run().await {
        Ok(_) => {
            tracing::debug!("Rollup execution complete. Shutting down.");
        }
        Err(e) => {
            tracing::error!(error = ?e, backtrace= e.backtrace().to_string(), "Rollup execution failed");
            exit(1);
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    prometheus_exporter::start(args.prometheus_exporter_bind.parse()?)
        .context("Prometheus exporter start failed")?;

    let rollup_config_path = args.rollup_config_path.as_str();

    let prover_config = parse_prover_config().expect("Failed to parse prover config");
    tracing::info!(?prover_config, "Running demo rollup with prover config");

    match args.da_layer {
        SupportedDaLayer::Mock => {
            let rollup = new_rollup_with_mock_da(
                &GenesisPaths::from_dir(&args.genesis_config_dir),
                rollup_config_path,
                prover_config,
            )
            .await
            .context("Failed to initialize MockDa rollup")?;
            rollup.run().await
        }
        SupportedDaLayer::Celestia => {
            let rollup = new_rollup_with_celestia_da(
                &GenesisPaths::from_dir(&args.genesis_config_dir),
                rollup_config_path,
                prover_config,
            )
            .await
            .context("Failed to initialize Celestia rollup")?;
            rollup.run().await
        }
    }
}

fn parse_prover_config() -> anyhow::Result<Option<RollupProverConfig>> {
    if let Some(value) = option_env!("SOV_PROVER_MODE") {
        let config = std::str::FromStr::from_str(value).map_err(|error| {
            tracing::error!(value, ?error, "Unknown `SOV_PROVER_MODE` value; aborting");
            error
        })?;
        #[cfg(debug_assertions)]
        {
            if config == RollupProverConfig::Prove {
                tracing::warn!(prover_config = ?config, "Given RollupProverConfig might cause slow rollup progression if not compiled in release mode.");
            }
        }
        Ok(Some(config))
    } else {
        Ok(None)
    }
}

async fn new_rollup_with_celestia_da(
    rt_genesis_paths: &GenesisPaths,
    rollup_config_path: &str,
    prover_config: Option<RollupProverConfig>,
) -> anyhow::Result<Rollup<CelestiaRollup<Native>, Native>> {
    debug!(config_path = rollup_config_path, "Starting Celestia rollup");

    let rollup_config: RollupConfig<Address<Sha256>, CelestiaService> =
        from_toml_path(rollup_config_path).with_context(|| {
            format!(
                "Failed to read rollup configuration from {}",
                rollup_config_path
            )
        })?;

    let celestia_rollup = CelestiaRollup::<Native>::default();
    celestia_rollup
        .create_new_rollup(rt_genesis_paths, rollup_config, prover_config)
        .await
}

async fn new_rollup_with_mock_da(
    rt_genesis_paths: &GenesisPaths,
    rollup_config_path: &str,
    prover_config: Option<RollupProverConfig>,
) -> anyhow::Result<Rollup<MockRollup<Native>, Native>> {
    debug!(
        config_path = rollup_config_path,
        "Starting rollup on mock DA"
    );

    let rollup_config: RollupConfig<Address<Sha256>, StorableMockDaService> =
        from_toml_path(rollup_config_path).with_context(|| {
            format!(
                "Failed to read rollup configuration from {}",
                rollup_config_path
            )
        })?;

    let mock_rollup = MockRollup::<Native>::default();
    mock_rollup
        .create_new_rollup(rt_genesis_paths, rollup_config, prover_config)
        .await
}