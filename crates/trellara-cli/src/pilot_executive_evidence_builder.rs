use std::path::Path;

use crate::{DatasetMode, PilotExecutiveEvidenceSummary, StreamConfig, TrellaraConfig};

impl PilotExecutiveEvidenceSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Self {
        let config_path = path.display().to_string();
        let local_stream = matches!(config.stream, StreamConfig::Local { .. });
        let partitioned = config.dataset.mode == DatasetMode::PartitionedScaleMode;
        let stream_kind = if local_stream { "local" } else { "kafka" }.to_string();
        let proof_commands =
            pilot_evidence_proof_commands(config, &config_path, local_stream, partitioned);
        let differentiated_controls = differentiated_controls(local_stream, partitioned);
        let live_evidence_required = live_evidence_required(partitioned);

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            config: config_path,
            north_star: "Trellara should make Postgres CDC something an enterprise can trust: every committed transaction is either durably moved, provably converged, or explicitly blocked with a safe recovery path.".to_string(),
            evaluation_thesis: "The pilot is successful when platform, data, and security leaders can review one package and see source safety, transaction-boundary proof, convergence evidence, and failure recovery for a real Postgres flow.".to_string(),
            transaction_boundary: config.explain(),
            stream_kind,
            kafka_required: !local_stream,
            proof_commands,
            differentiated_controls,
            live_evidence_required,
            executive_decision: "Advance only when the package proves a verified flow under the customer's source constraints; otherwise Trellara should name the blocker and the exact recovery or configuration step.".to_string(),
        }
    }
}

fn pilot_evidence_proof_commands(
    config: &TrellaraConfig,
    config_path: &str,
    local_stream: bool,
    partitioned: bool,
) -> Vec<String> {
    let mut commands = vec![
        format!("trellara check --config {config_path} --format text"),
        format!("trellara contract-test --config {config_path}"),
        format!("trellara snapshot --config {config_path} --run-id pilot-snapshot-1"),
        format!("trellara status --config {config_path} --view report --format text"),
        format!("trellara status --config {config_path} --view metrics"),
        "trellara chaos run".to_string(),
    ];
    if config.target.is_some() {
        commands.push(format!("trellara verify --config {config_path}"));
    }
    if local_stream {
        commands.push(format!(
            "trellara stream inspect-local --config {config_path}"
        ));
    }
    if partitioned {
        commands.push(format!(
            "trellara partition-watermarks --config {config_path}"
        ));
    }
    commands
}

fn differentiated_controls(local_stream: bool, partitioned: bool) -> Vec<String> {
    let mut controls = vec![
        "read-only source-safety scoring before CDC creates operational pressure on the source".to_string(),
        "transaction-boundary proof tracks source durability, target checkpoint catch-up, and manifest barriers explicitly".to_string(),
        "quarantine fails closed on target contract or zero-row apply divergence instead of silently advancing checkpoints".to_string(),
        "verification, reseed, repair-plan, diagnostics, and metrics are first-class operator surfaces".to_string(),
        "nightly deterministic failure matrix turns correctness claims into repeatable evidence".to_string(),
    ];
    if local_stream {
        controls.push(
            "brokerless local stream lets evaluators prove the loop without adopting Kafka or Redpanda"
                .to_string(),
        );
    }
    if partitioned {
        controls.push(
            "partitioned scale mode preserves transaction identity while exposing global visibility only after every partition watermark is complete"
                .to_string(),
        );
    }
    controls
}

fn live_evidence_required(partitioned: bool) -> Vec<String> {
    let mut required = vec![
        "source-safety must show no critical slot, WAL retention, replica identity, or subscription conflict blockers".to_string(),
        "snapshot handoff must record complete table copy progress and a durable handoff watermark".to_string(),
        "verify must report converged=true and checksum_status=match for the selected tables".to_string(),
        "failure drill must demonstrate quarantine, repair-plan, and replay-ready recovery without advancing the target checkpoint incorrectly".to_string(),
        "status metrics must expose flow readiness, transaction-boundary status, alert count, and quarantine reason when blocked".to_string(),
    ];
    if partitioned {
        required.push(
            "partition-watermarks must show every partition complete before global current-state visibility advances"
                .to_string(),
        );
    }
    required
}
