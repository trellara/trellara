use crate::{DatasetMode, PilotGuidePhase, StreamConfig, TrellaraConfig};

pub(crate) fn pilot_guide_phases(
    config: &TrellaraConfig,
    config_path: &str,
) -> Vec<PilotGuidePhase> {
    let mut phases = vec![
        PilotGuidePhase::new(
            1,
            "source safety",
            format!("trellara check --config {config_path}"),
            "replication slot, WAL retention, failover-slot posture, table identity, and subscription conflict risks are explicit",
        ),
        PilotGuidePhase::new(
            2,
            "contract preflight",
            format!("trellara contract-test --config {config_path}"),
            "source and target schema contracts fail closed before CDC starts",
        ),
        PilotGuidePhase::new(
            3,
            "snapshot handoff",
            format!("trellara snapshot --config {config_path} --run-id pilot-snapshot-1"),
            "initial copy records a durable handoff boundary before stream replay is trusted",
        ),
    ];

    if matches!(config.stream, StreamConfig::Local { .. }) && config.target.is_some() {
        phases.push(PilotGuidePhase::new(
            4,
            "brokerless verified loop",
            format!(
                "trellara run --local --verify --format text --config {config_path} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100"
            ),
            "relay, apply, and verification run against the embedded durable local stream without Kafka",
        ));
    } else {
        phases.push(PilotGuidePhase::new(
            4,
            "streamed relay",
            format!("trellara relay --config {config_path}"),
            "pgoutput commits are durably published before source feedback advances",
        ));
        if config.target.is_some() {
            phases.push(PilotGuidePhase::new(
                5,
                "target apply",
                format!("trellara apply --config {config_path}"),
                "target checkpoint and dedup state advance in the same transaction as apply",
            ));
        }
    }

    if config.target.is_some() {
        phases.push(PilotGuidePhase::new(
            phases.len() + 1,
            "convergence verification",
            format!("trellara verify --config {config_path}"),
            "source and target watermarks plus checksums prove the selected tables converged",
        ));
    }
    phases.push(PilotGuidePhase::new(
        phases.len() + 1,
        "operator proof package",
        format!("trellara status --config {config_path} --view diagnostics --format text"),
        "report, alerts, metrics, latest failure, and repair-plan output are bundled for review",
    ));

    phases
}

pub(crate) fn pilot_evidence_commands(config: &TrellaraConfig, config_path: &str) -> Vec<String> {
    let mut evidence_commands = vec![
        format!("trellara status --config {config_path} --view report --format text"),
        format!("trellara status --config {config_path} --view dashboard --format text"),
        format!("trellara status --config {config_path} --view metrics"),
        format!("trellara status --config {config_path} --view diagnostics --format text"),
        "trellara chaos run".to_string(),
    ];
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        evidence_commands.push(format!(
            "trellara partition-watermarks --config {config_path}"
        ));
    }

    evidence_commands
}

pub(crate) fn pilot_large_transaction_evidence(
    config: &TrellaraConfig,
    config_path: &str,
) -> Vec<String> {
    let mut large_transaction_evidence = vec![
        format!("trellara quickstart --config {config_path} --check"),
        format!("trellara contract-test --config {config_path}"),
        "trellara chaos run".to_string(),
        format!("trellara status --config {config_path} --view report --format text"),
    ];
    if matches!(config.stream, StreamConfig::Local { .. }) {
        large_transaction_evidence.push(format!(
            "trellara stream inspect-local --config {config_path}"
        ));
    }

    large_transaction_evidence
}

pub(crate) fn pilot_failure_drill(config: &TrellaraConfig, config_path: &str) -> Vec<String> {
    let mut failure_drill = vec![
        format!("trellara quarantine list --config {config_path}"),
        format!("trellara repair-plan --config {config_path}"),
        format!(
            "trellara quarantine replay-ready --config {config_path} --transaction-id <tx> --commit-lsn <lsn>"
        ),
    ];
    if matches!(config.stream, StreamConfig::Local { .. }) {
        failure_drill.push(format!(
            "trellara stream inspect-local --config {config_path}"
        ));
    }

    failure_drill
}
