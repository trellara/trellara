use crate::{
    enterprise_run_proof_command, DatasetMode, PilotScorecardStatus, PilotScorecardSummary,
    TrellaraConfig,
};

pub(crate) fn enterprise_proof_commands(
    config: &TrellaraConfig,
    config_path: &str,
    local_stream: bool,
) -> Vec<String> {
    let mut proof_commands = vec![
        format!("trellara check --config {config_path} --format text"),
        format!("trellara preflight --config {config_path}"),
        format!("trellara contract-test --config {config_path}"),
        enterprise_run_proof_command(config_path, local_stream),
        format!("trellara status --config {config_path} --view diagnostics --format text"),
        format!("trellara pilot-package --config {config_path}"),
    ];
    if local_stream {
        proof_commands.push(format!(
            "trellara stream inspect-local --config {config_path}"
        ));
    }
    if is_partitioned(config) {
        proof_commands.push(format!(
            "trellara partition-watermarks --config {config_path}"
        ));
    }
    proof_commands
        .push("trellara inspect-transaction --file <envelope.pb> --format text".to_string());
    proof_commands
}

pub(crate) fn enterprise_differentiators(config: &TrellaraConfig) -> Vec<String> {
    let mut differentiators = vec![
        "verified replication posture: source safety, contract preflight, convergence verification, and recovery evidence are product surfaces".to_string(),
        "transaction-first protocol: every CDC envelope is inspectable by transaction, manifest boundary_mode, checksum, and commit-marker proof".to_string(),
        "enterprise failure posture: target errors quarantine instead of corrupting checkpoints, with repair-plan and replay-ready commands".to_string(),
        "broker flexibility: the same flow can evaluate locally without Kafka or run through Kafka/Redpanda when the customer already standardizes on it".to_string(),
    ];
    if is_partitioned(config) {
        differentiators.push(
            "partitioned scale is an explicit contract, not a vague performance toggle: manifests preserve completeness while consumers choose barrier-aware or partition-local visibility".to_string(),
        );
    } else if config.dataset.strict_chunking.is_some() {
        differentiators.push(
            "bounded strict mode handles large transactions through strict chunk manifests and commit markers without weakening source transaction order".to_string(),
        );
    }
    differentiators
}

pub(crate) fn enterprise_live_evidence_required(
    config: &TrellaraConfig,
    local_stream: bool,
) -> Vec<String> {
    let mut live_evidence_required = vec![
        "read-only source-safety output with no critical slot, WAL retention, replica identity, or failover-slot blockers".to_string(),
        "contract-test output proving configured source tables, target compatibility, pinned schema expectations, and transaction-boundary mode".to_string(),
        "run or relay/apply output proving source feedback only advances after durable publish and target checkpoint safety".to_string(),
        "verify output with converged=true and checksum_status=match for pilot tables".to_string(),
        "status diagnostics showing alerts, recovery actions, proof checks, latest failure, and transaction-boundary status".to_string(),
    ];
    if local_stream {
        live_evidence_required.push(
            "local stream inspection showing durable segment health, cursor offsets, barrier topics, and no torn-tail corruption".to_string(),
        );
    }
    if is_partitioned(config) {
        live_evidence_required.push(
            "partition-watermarks output proving every partition has reached the global visibility boundary before atomic exposure".to_string(),
        );
    }
    live_evidence_required
}

pub(crate) fn enterprise_blockers(
    scorecard: &PilotScorecardSummary,
    has_target: bool,
) -> Vec<String> {
    let mut blockers = scorecard
        .gates
        .iter()
        .filter(|gate| gate.status == PilotScorecardStatus::Blocked)
        .map(|gate| format!("{}: {}", gate.code, gate.evidence))
        .collect::<Vec<_>>();
    if !has_target {
        blockers.push(
            "target.database_url is required for an enterprise Postgres-to-Postgres pilot"
                .to_string(),
        );
    }
    blockers
}

pub(crate) fn enterprise_next_commands(
    config: &TrellaraConfig,
    config_path: &str,
    local_stream: bool,
    has_target: bool,
) -> Vec<String> {
    let mut next_commands = vec![
        format!("trellara check --config {config_path} --format text"),
        format!("trellara contract-test --config {config_path}"),
    ];
    if local_stream && has_target {
        next_commands.push(format!(
            "trellara run --local --verify --format text --config {config_path} --snapshot-run-id enterprise-eval-snapshot --max-transactions 100 --max-messages 100"
        ));
    } else if has_target {
        next_commands.push(format!("trellara relay --config {config_path}"));
        next_commands.push(format!("trellara apply --config {config_path}"));
        next_commands.push(format!("trellara verify --config {config_path}"));
    }
    if is_partitioned(config) {
        next_commands.push(format!(
            "trellara partition-watermarks --config {config_path}"
        ));
    }
    next_commands.push(format!("trellara pilot-package --config {config_path}"));
    next_commands
}

fn is_partitioned(config: &TrellaraConfig) -> bool {
    config.dataset.mode == DatasetMode::PartitionedScaleMode
}
