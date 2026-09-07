use std::path::Path;

use crate::{
    consistency_consumer_visibility_contract, consistency_invariants,
    consistency_partition_contract, consistency_replay_contract, consistency_run_proof_command,
    consistency_source_ack_contract, consistency_source_capture_contract, consistency_stream_kind,
    consistency_transaction_boundary_contract, consistency_transport_durability_contract,
    ConsistencyContractSummary, DatasetMode, Result, StreamConfig, TrellaraConfig,
};
use trellara_stream::TopicLayout;

impl ConsistencyContractSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Result<Self> {
        let config_path = path.display().to_string();
        let topics = config.replay_redelivery_topics()?;
        let layout = TopicLayout::new(&config.source.id, &config.dataset.id)?;
        let selected_consumer_mode = match config.dataset.mode {
            DatasetMode::StrictTransactionOrder => "exact_transaction".to_string(),
            DatasetMode::PartitionedScaleMode => "barrier_aware".to_string(),
        };
        let partition_contract = consistency_partition_contract(config, &layout);

        let mut proof_commands = vec![
            format!("trellara check --config {config_path} --format text"),
            format!("trellara contract-test --config {config_path}"),
            format!("trellara snapshot --config {config_path} --run-id consistency-contract"),
            consistency_run_proof_command(
                &config_path,
                matches!(config.stream, StreamConfig::Local { .. }),
            ),
            format!("trellara verify --config {config_path}"),
            format!("trellara status --config {config_path} --view diagnostics --format text"),
            "trellara inspect-transaction --file <envelope.pb> --format text".to_string(),
        ];
        if matches!(config.stream, StreamConfig::Local { .. }) {
            proof_commands.push(format!(
                "trellara stream inspect-local --config {config_path}"
            ));
            proof_commands.push(format!(
                "trellara stream locate-local --config {config_path} --transaction-id <tx> --commit-lsn <lsn>"
            ));
        }
        if config.dataset.mode == DatasetMode::PartitionedScaleMode {
            if matches!(config.stream, StreamConfig::Local { .. }) {
                proof_commands.push(format!(
                    "trellara stream reconstruct-local --config {config_path} --transaction-id <tx> --commit-lsn <lsn>"
                ));
            }
            proof_commands.push(format!(
                "trellara partition-watermarks --config {config_path}"
            ));
        }

        let invariants = consistency_invariants(config, &config_path);
        let mut next_commands = vec![
            format!("trellara consistency --config {config_path} --format text"),
            format!("trellara semantics --config {config_path}"),
            format!("trellara pilot-package --config {config_path}"),
        ];
        if config.dataset.mode == DatasetMode::PartitionedScaleMode {
            next_commands.insert(
                1,
                format!("trellara partition-watermarks --config {config_path}"),
            );
        }

        Ok(Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            config: config_path,
            mode: config.status_mode(),
            selected_consumer_mode,
            stream_kind: consistency_stream_kind(&config.stream),
            topics,
            source_capture_contract: consistency_source_capture_contract(config),
            transaction_boundary_contract: consistency_transaction_boundary_contract(config),
            source_ack_contract: consistency_source_ack_contract(config),
            snapshot_handoff_contract:
                "initial copy is tied to the pgoutput slot consistent LSN; CDC may resume only after copied tables have a durable handoff watermark and verification evidence"
                    .to_string(),
            transport_durability_contract: consistency_transport_durability_contract(config),
            consumer_visibility_contract: consistency_consumer_visibility_contract(config),
            target_checkpoint_contract:
                "target checkpoints advance only after the complete transaction boundary is applied, deduplicated, and any checksum or row-count verification evidence is recorded"
                    .to_string(),
            replay_contract: consistency_replay_contract(config),
            reseed_contract:
                "reseed creates a fresh snapshot handoff for the affected scope and requires verify convergence before the flow is trusted again"
                    .to_string(),
            partition_contract,
            lake_visibility_contract:
                "lake raw CDC and epoch metadata become visible only after the Trellara commit LSN or partition watermark contract is satisfied; current-state and SCD2 are Spark-derived from completed epochs"
                    .to_string(),
            invariants,
            proof_commands,
            next_commands,
        })
    }
}
