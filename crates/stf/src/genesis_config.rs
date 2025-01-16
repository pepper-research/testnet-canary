use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use sov_accounts::AccountConfig;
use sov_attester_incentives::AttesterIncentivesConfig;
use sov_bank::BankConfig;
use sov_chain_state::ChainStateConfig;
use sov_modules_api::Spec;
use sov_modules_stf_blueprint::Runtime as RuntimeTrait;
use sov_paymaster::PaymasterConfig;
use sov_prover_incentives::ProverIncentivesConfig;
use sov_sequencer_registry::SequencerConfig;
use spicenet_time::TimeConfig;
use capsule::CapsuleConfig;
use spicenet_aaob::AAOBConfig;
use spicenet_risk::genesis::RiskModuleConfig;
use lut::LookupTableConfig;

use super::GenesisConfig;
use crate::Runtime;

/// Paths to genesis files.
#[derive(Clone, Debug)]
pub struct GenesisPaths {
    /// Accounts genesis path.
    pub accounts_genesis_path: PathBuf,
    /// Bank genesis path.
    pub bank_genesis_path: PathBuf,
    /// Sequencer Registry genesis path.
    pub sequencer_genesis_path: PathBuf,
    /// Attester Incentives genesis path.
    pub attester_incentives_genesis_path: PathBuf,
    /// Prover Incentives genesis path.
    pub prover_incentives_genesis_path: PathBuf,
    /// Chain State genesis path.
    pub chain_state_genesis_path: PathBuf,
    /// Paymaster genesis path.
    pub paymaster_genesis_path: PathBuf,
    /// Time genesis path.
    pub time_genesis_path: PathBuf,
    /// Capsule genesis path.
    pub capsule_genesis_path: PathBuf,
    /// AAOB genesis path.
    pub aaob_genesis_path: PathBuf,
    /// Risk module genesis path.
    pub risk_genesis_path: PathBuf,
    /// Lookup Table genesis path.
    pub lut_genesis_path: PathBuf,
}

impl GenesisPaths {
    /// Creates a new [`RuntimeTrait::GenesisConfig`] from the files contained in
    /// the given directory.
    ///
    /// Take a look at the contents of the `test_data` directory to see the
    /// expected files.
    pub fn from_dir(dir: impl AsRef<Path>) -> Self {
        Self {
            accounts_genesis_path: dir.as_ref().join("accounts.json"),
            bank_genesis_path: dir.as_ref().join("bank.json"),
            sequencer_genesis_path: dir.as_ref().join("sequencer_registry.json"),
            attester_incentives_genesis_path: dir.as_ref().join("attester_incentives.json"),
            prover_incentives_genesis_path: dir.as_ref().join("prover_incentives.json"),
            chain_state_genesis_path: dir.as_ref().join("chain_state.json"),
            paymaster_genesis_path: dir.as_ref().join("paymaster.json"),
            time_genesis_path: dir.as_ref().join("time.json"),
            capsule_genesis_path: dir.as_ref().join("capsule.json"),
            aaob_genesis_path: dir.as_ref().join("aaob.json"),
            risk_genesis_path: dir.as_ref().join("risk.json"),
            lut_genesis_path: dir.as_ref().join("lut.json"),
        }
    }
}

/// Creates a new [`GenesisConfig`] from the files contained in the given
/// directory.
pub fn create_genesis_config<S: Spec>(
    genesis_paths: &GenesisPaths,
) -> anyhow::Result<<Runtime<S> as RuntimeTrait<S>>::GenesisConfig> {
    let accounts_config: AccountConfig<S> =
        read_genesis_json(&genesis_paths.accounts_genesis_path)?;
    let bank_config: BankConfig<S> = read_genesis_json(&genesis_paths.bank_genesis_path)?;
    let sequencer_registry_config: SequencerConfig<S> =
        read_genesis_json(&genesis_paths.sequencer_genesis_path)?;
    let attester_incentives_config: AttesterIncentivesConfig<S> =
        read_genesis_json(&genesis_paths.attester_incentives_genesis_path)?;

    let prover_incentives_config: ProverIncentivesConfig<S> =
        read_genesis_json(&genesis_paths.prover_incentives_genesis_path)?;

    let nonces_config = ();

    let chain_state_config: ChainStateConfig<S> =
        read_genesis_json(&genesis_paths.chain_state_genesis_path)?;

    let blob_storage_config = ();

    let paymaster_config: PaymasterConfig<S> =
        read_genesis_json(&genesis_paths.paymaster_genesis_path)?;

    let time_config: TimeConfig<S> = read_genesis_json(&genesis_paths.time_genesis_path)?;

    let capsule_config: CapsuleConfig = read_genesis_json(&genesis_paths.capsule_genesis_path)?;

    let aaob_config: AAOBConfig<S> = read_genesis_json(&genesis_paths.aaob_genesis_path)?;

    let risk_config: RiskModuleConfig = read_genesis_json(&genesis_paths.risk_genesis_path)?;

    let lut_config: LookupTableConfig<S> = read_genesis_json(&genesis_paths.lut_genesis_path)?;

    Ok(GenesisConfig::new(
        accounts_config,
        nonces_config,
        bank_config,
        sequencer_registry_config,
        attester_incentives_config,
        prover_incentives_config,
        chain_state_config,
        blob_storage_config,
        paymaster_config,
        time_config,
        capsule_config,
        aaob_config,
        risk_config,
        lut_config,
    ))
}

fn read_genesis_json<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}
