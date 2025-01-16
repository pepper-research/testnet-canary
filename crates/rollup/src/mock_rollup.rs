use std::sync::Arc;

use async_trait::async_trait;
use sov_db::ledger_db::LedgerDb;
use sov_db::storage_manager::NativeStorageManager;
use sov_mock_da::storable::service::StorableMockDaService;
use sov_mock_da::MockDaSpec;
use sov_mock_zkvm::{MockCodeCommitment, MockZkvm, MockZkvmHost};
use sov_modules_api::default_spec::DefaultSpec;
use sov_modules_api::execution_mode::{ExecutionMode, Native};
use sov_modules_api::rest::StateUpdateReceiver;
use sov_modules_api::{CryptoSpec, RuntimeEndpoints, Spec, SyncStatus, ZkVerifier};
use sov_modules_rollup_blueprint::pluggable_traits::PluggableSpec;
use sov_modules_rollup_blueprint::proof_serializer::SovApiProofSerializer;
use sov_modules_rollup_blueprint::{FullNodeBlueprint, RollupBlueprint, SequencerCreationReceipt};
use sov_risc0_adapter::host::Risc0Host;
use sov_risc0_adapter::Risc0;
use sov_rollup_interface::zk::aggregated_proof::CodeCommitment;
use sov_sequencer::SequenceNumberProvider;
use sov_state::{DefaultStorageSpec, ProverStorage, Storage};
use sov_stf_runner::processes::{ParallelProverService, ProverService, RollupProverConfig};
use sov_stf_runner::RollupConfig;
use spicenet_stf::runtime::Runtime;

/// Rollup with MockDa
#[derive(Default)]
pub struct MockRollup<M> {
    phantom: std::marker::PhantomData<M>,
}

type MockRollupSpec<M> = DefaultSpec<MockDaSpec, Risc0, MockZkvm, M>;

impl<M: ExecutionMode> RollupBlueprint<M> for MockRollup<M>
where
    MockRollupSpec<M>: PluggableSpec,
{
    type Spec = MockRollupSpec<M>;
    type Runtime = Runtime<Self::Spec>;
}

#[async_trait]
impl FullNodeBlueprint<Native> for MockRollup<Native> {
    type DaService = StorableMockDaService;

    type StorageManager = NativeStorageManager<
        MockDaSpec,
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
        let endpoints = sov_modules_rollup_blueprint::register_endpoints::<Self, Native>(
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
        shutdown_receiver: tokio::sync::watch::Receiver<()>,
    ) -> Self::DaService {
        StorableMockDaService::from_config(rollup_config.da.clone(), shutdown_receiver).await
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
            let elf = std::fs::read(risc0::MOCK_DA_PATH)
                .unwrap_or_else(|e| {
                    panic!(
                        "Could not read guest elf file from `{}`. {}",
                        risc0::MOCK_DA_PATH,
                        e
                    )
                })
                .leak();
            Risc0Host::new(elf)
        };

        let outer_vm = MockZkvmHost::new_non_blocking();
        let da_verifier = Default::default();

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
