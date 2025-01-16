use std::sync::Arc;

use async_trait::async_trait;
use sov_celestia_adapter::verifier::{CelestiaSpec, CelestiaVerifier, RollupParams};
use sov_celestia_adapter::CelestiaService;
use sov_db::ledger_db::LedgerDb;
use sov_db::storage_manager::NativeStorageManager;
use sov_mock_zkvm::{MockCodeCommitment, MockZkvm, MockZkvmHost};
use sov_modules_api::default_spec::DefaultSpec;
use sov_modules_api::execution_mode::{ExecutionMode, Native};
use sov_modules_api::rest::StateUpdateReceiver;
use sov_modules_api::{CryptoSpec, RuntimeEndpoints, Spec, SyncStatus, ZkVerifier};
use sov_modules_rollup_blueprint::pluggable_traits::PluggableSpec;
use sov_modules_rollup_blueprint::proof_serializer::SovApiProofSerializer;
use sov_modules_rollup_blueprint::{
    FullNodeBlueprint, RollupBlueprint, SequencerCreationReceipt, WalletBlueprint,
};
use sov_risc0_adapter::host::Risc0Host;
use sov_risc0_adapter::Risc0;
use sov_rollup_interface::zk::aggregated_proof::CodeCommitment;
use sov_sequencer::SequenceNumberProvider;
use sov_state::{DefaultStorageSpec, ProverStorage, Storage};
use sov_stf_runner::processes::{ParallelProverService, ProverService, RollupProverConfig};
use sov_stf_runner::RollupConfig;
use spicenet_stf::runtime::Runtime;

use crate::{ROLLUP_BATCH_NAMESPACE, ROLLUP_PROOF_NAMESPACE};

/// Rollup with CelestiaDa
#[derive(Default)]
pub struct CelestiaRollup<M> {
    phantom: std::marker::PhantomData<M>,
}

type CelestiaRollupSpec<M> = DefaultSpec<CelestiaSpec, Risc0, MockZkvm, M>;

impl<M: ExecutionMode> RollupBlueprint<M> for CelestiaRollup<M>
where
    CelestiaRollupSpec<M>: PluggableSpec,
{
    type Spec = CelestiaRollupSpec<M>;
    type Runtime = Runtime<Self::Spec>;
}

#[async_trait]
impl FullNodeBlueprint<Native> for CelestiaRollup<Native> {
    type DaService = CelestiaService;

    type StorageManager = NativeStorageManager<
        CelestiaSpec,
        ProverStorage<DefaultStorageSpec<<<Self::Spec as Spec>::CryptoSpec as CryptoSpec>::Hasher>>,
    >;

    type ProverService = ParallelProverService<
        <Self::Spec as Spec>::Address,
        <<Self::Spec as Spec>::Storage as Storage>::Root,
        <<Self::Spec as Spec>::Storage as Storage>::Witness,
        Self::DaService,
        <Self::Spec as Spec>::InnerZkvm,
        <Self::Spec as Spec>::OuterZkvm,
    >;

    type ProofSerializer = SovApiProofSerializer<Self::Spec>;

    fn create_outer_code_commitment(
        &self,
    ) -> <<Self::ProverService as ProverService>::Verifier as ZkVerifier>::CodeCommitment {
        MockCodeCommitment::default()
    }

    async fn create_endpoints(
        &self,
        state_update_receiver: StateUpdateReceiver<<Self::Spec as Spec>::Storage>,
        sync_status_receiver: tokio::sync::watch::Receiver<SyncStatus>,
        ledger_db: &LedgerDb,
        sequencer: &SequencerCreationReceipt<Self::Spec>,
        _da_service: &Self::DaService,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
    ) -> anyhow::Result<RuntimeEndpoints> {
        let endpoints = sov_modules_rollup_blueprint::register_endpoints::<Self, _>(
            state_update_receiver.clone(),
            sync_status_receiver,
            ledger_db,
            sequencer,
            rollup_config,
        )
        .await?;

        // TODO: Add issue for Sequencer level RPC injection:
        //   https://github.com/Sovereign-Labs/sovereign-sdk-wip/issues/366
        // crate::eth::register_ethereum::<Self::Spec, Self::DaService, Self::Runtime>(
        //     da_service.clone(),
        //     state_update_receiver,
        //     &mut endpoints.jsonrpsee_module,
        // )?;

        Ok(endpoints)
    }

    async fn create_da_service(
        &self,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
        _shutdown_receiver: tokio::sync::watch::Receiver<()>,
    ) -> Self::DaService {
        CelestiaService::new(
            rollup_config.da.clone(),
            RollupParams {
                rollup_batch_namespace: ROLLUP_BATCH_NAMESPACE,
                rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
            },
        )
        .await
    }

    async fn create_prover_service(
        &self,
        prover_config: RollupProverConfig,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
        _da_service: &Self::DaService,
    ) -> Self::ProverService {
        let inner_vm = if let RollupProverConfig::Skip = prover_config {
            Risc0Host::new(b"")
        } else {
            let elf = std::fs::read(risc0::ROLLUP_PATH)
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not read guest elf file from `{}`. {}",
                        risc0::ROLLUP_PATH,
                        e
                    )
                })
                .leak();
            Risc0Host::new(elf)
        };

        let outer_vm = MockZkvmHost::new_non_blocking();

        let da_verifier = CelestiaVerifier {
            rollup_batch_namespace: ROLLUP_BATCH_NAMESPACE,
            rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
        };

        ParallelProverService::new_with_default_workers(
            inner_vm,
            outer_vm,
            da_verifier,
            prover_config,
            CodeCommitment::default(),
            rollup_config.proof_manager.prover_address,
        )
    }

    fn create_storage_manager(
        &self,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
    ) -> anyhow::Result<Self::StorageManager> {
        NativeStorageManager::new(&rollup_config.storage.path)
    }

    fn create_proof_serializer(
        &self,
        _rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
        sequence_number_provider: Option<Arc<dyn SequenceNumberProvider>>,
    ) -> anyhow::Result<Self::ProofSerializer> {
        Ok(Self::ProofSerializer::new(sequence_number_provider))
    }
}

impl WalletBlueprint<Native> for CelestiaRollup<Native> {}
