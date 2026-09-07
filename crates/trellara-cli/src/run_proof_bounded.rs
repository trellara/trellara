use std::path::Path;

use crate::{
    parallel_replay_contract, DatasetMode, RunProofGate, RunProofStatus, SourceCaptureKind,
    TrellaraConfig,
};

pub(crate) fn bounded_large_transaction_run_proof(
    config_path: &Path,
    config: &TrellaraConfig,
) -> RunProofGate {
    let config_display = config_path.display().to_string();
    let spill_threshold = config
        .source
        .stream_spill_threshold_changes
        .unwrap_or(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES);
    let pgoutput_streaming = config.source.capture == SourceCaptureKind::PgOutput
        && config.source.pgoutput.protocol_version >= 2
        && config.source.pgoutput.streaming;
    let spill_configured = spill_threshold > 0;
    let boundary = large_transaction_boundary_evidence(config);
    let status = if pgoutput_streaming && spill_configured && boundary.is_some() {
        RunProofStatus::Verified
    } else {
        RunProofStatus::AtRisk
    };
    let boundary = boundary.unwrap_or_else(LargeTransactionBoundaryEvidence::missing);

    RunProofGate {
        code: "bounded_large_transaction_capture".to_string(),
        status,
        evidence: format!(
            "mode={} capture={} pgoutput.protocol_version={} pgoutput.streaming={} spill_threshold_changes={} boundary={} bounded_memory_contract={} visibility_contract={} parallel_replay_contract={}",
            config.dataset.mode,
            config.source.capture.expected_plugin(),
            config.source.pgoutput.protocol_version,
            config.source.pgoutput.streaming,
            spill_threshold,
            boundary.boundary,
            boundary.bounded_memory_contract,
            boundary.visibility_contract,
            parallel_replay_contract(&config.dataset.mode.to_string())
        ),
        proof_command: format!("trellara mvp-check --config {config_display} --format text"),
    }
}

struct LargeTransactionBoundaryEvidence {
    boundary: String,
    bounded_memory_contract: &'static str,
    visibility_contract: &'static str,
}

impl LargeTransactionBoundaryEvidence {
    fn missing() -> Self {
        Self {
            boundary: "missing_manifest_or_chunk_boundary".to_string(),
            bounded_memory_contract: "not_proven",
            visibility_contract: "not_proven",
        }
    }
}

fn large_transaction_boundary_evidence(
    config: &TrellaraConfig,
) -> Option<LargeTransactionBoundaryEvidence> {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => {
            config
                .dataset
                .strict_chunking
                .as_ref()
                .map(|strict_chunking| LargeTransactionBoundaryEvidence {
                    boundary: format!(
                        "strict_chunking.max_changes_per_chunk={}",
                        strict_chunking.max_changes_per_chunk
                    ),
                    bounded_memory_contract: "spill_then_strict_chunk_manifest",
                    visibility_contract: "commit_marker_waits_for_complete_strict_chunk_set",
                })
        }
        DatasetMode::PartitionedScaleMode => {
            config
                .dataset
                .partition
                .as_ref()
                .map(|partition| LargeTransactionBoundaryEvidence {
                    boundary: format!(
                        "partition_manifest_barrier.partition_count={} key_column={}",
                        partition.partition_count, partition.key_column
                    ),
                    bounded_memory_contract: "spill_then_partition_manifest",
                    visibility_contract:
                        "global_visibility_waits_for_manifest_commit_marker_and_all_partitions",
                })
        }
    }
}
