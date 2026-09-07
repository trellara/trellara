use std::path::Path;

use crate::fleet_types::{FleetConvergenceGate, FleetConvergenceGateStatus};
use crate::{DatasetMode, StreamConfig, TrellaraConfig};

pub(crate) fn fleet_convergence_gates(
    config: &TrellaraConfig,
    path: &Path,
) -> Vec<FleetConvergenceGate> {
    let config_display = path.display();
    let mut gates = vec![
        needs_live_evidence(
            "source_safety",
            "source slot, WAL retention, replica identity, and failover-slot posture have no critical blockers",
            format!("trellara check --config {config_display} --format text"),
        ),
        needs_live_evidence(
            "contract_preflight",
            "source and target schemas, table policies, and transaction-boundary contract are accepted before CDC starts",
            format!("trellara contract-test --config {config_display}"),
        ),
        needs_live_evidence(
            "transaction_boundary",
            match config.dataset.mode {
                DatasetMode::StrictTransactionOrder => {
                    if config.dataset.strict_chunking.is_some() {
                        "strict chunked visibility waits for every chunk, manifest, and commit marker"
                    } else {
                        "strict visibility preserves one complete source transaction per envelope"
                    }
                }
                DatasetMode::PartitionedScaleMode => {
                    "partitioned visibility preserves transaction identity through manifest and commit marker barriers"
                }
            },
            format!("trellara status --config {config_display} --view report --format text"),
        ),
        if config.target.is_some() {
            needs_live_evidence(
                "target_convergence",
                "verify reports converged=true and checksum_status=match for every configured table",
                format!("trellara verify --config {config_display}"),
            )
        } else {
            blocked_by_config(
                "target_convergence",
                "target.database_url is missing, so row-count and checksum convergence cannot be proven",
                format!("trellara verify --config {config_display}"),
            )
        },
        needs_live_evidence(
            "operator_report",
            "status report shows source and target checkpoints, checksum status, latest failure, and recovery actions",
            format!("trellara status --config {config_display} --view report --format text"),
        ),
    ];

    if matches!(config.stream, StreamConfig::Local { .. }) {
        gates.push(needs_live_evidence(
            "local_stream_durability",
            "local segment log has healthy topics, cursors, and barrier-topic posture before declaring brokerless convergence",
            format!("trellara stream inspect-local --config {config_display}"),
        ));
    }
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        gates.push(needs_live_evidence(
            "partition_watermarks",
            "all partition checkpoints are complete before exposing a global current-state view",
            format!("trellara partition-watermarks --config {config_display}"),
        ));
    }

    gates
}

fn needs_live_evidence(
    code: impl Into<String>,
    evidence: impl Into<String>,
    proof_command: impl Into<String>,
) -> FleetConvergenceGate {
    FleetConvergenceGate {
        code: code.into(),
        status: FleetConvergenceGateStatus::NeedsLiveEvidence,
        evidence: evidence.into(),
        proof_command: proof_command.into(),
    }
}

fn blocked_by_config(
    code: impl Into<String>,
    evidence: impl Into<String>,
    proof_command: impl Into<String>,
) -> FleetConvergenceGate {
    FleetConvergenceGate {
        code: code.into(),
        status: FleetConvergenceGateStatus::BlockedByConfig,
        evidence: evidence.into(),
        proof_command: proof_command.into(),
    }
}

pub(crate) fn fleet_transaction_boundary(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => {
            if let Some(strict_chunking) = &config.dataset.strict_chunking {
                format!(
                    "strict chunked: one source transaction becomes chunks of at most {} changes, with visibility after manifest plus commit marker",
                    strict_chunking.max_changes_per_chunk
                )
            } else {
                "strict: one source transaction stays one ordered envelope".to_string()
            }
        }
        DatasetMode::PartitionedScaleMode => {
            let partition = config
                .dataset
                .partition
                .as_ref()
                .expect("validated partition settings");
            format!(
                "partitioned: {} partitions by {}, globally visible after manifest, commit marker, and complete partition watermarks",
                partition.partition_count, partition.key_column
            )
        }
    }
}
