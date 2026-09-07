use crate::{PilotScorecardGate, PilotScorecardStatus};

pub(crate) fn brokerless_evaluation_gate(
    config_path: &str,
    local_stream: bool,
) -> PilotScorecardGate {
    let status = if local_stream {
        PilotScorecardStatus::ConfigurationReady
    } else {
        PilotScorecardStatus::EnvironmentSpecific
    };
    let evidence = if local_stream {
        "stream.kind is local, so the pilot can run with the embedded durable stream".to_string()
    } else {
        "stream.kind is Kafka; this is acceptable when the customer already operates Kafka or Redpanda"
            .to_string()
    };

    PilotScorecardGate::new(
        "brokerless_evaluation",
        "no-broker trial path",
        status,
        evidence,
        format!("trellara quickstart --config {config_path} --check"),
        "design partners can evaluate without adopting Kafka unless their environment explicitly chooses Kafka",
    )
}

pub(crate) fn verified_apply_gate(
    config_path: &str,
    local_stream: bool,
    has_target: bool,
) -> PilotScorecardGate {
    PilotScorecardGate::new(
        "verified_apply",
        "target apply and convergence verification",
        if has_target {
            PilotScorecardStatus::NeedsLiveEvidence
        } else {
            PilotScorecardStatus::Blocked
        },
        if has_target {
            "target.database_url is configured for apply and checksum verification".to_string()
        } else {
            "target.database_url is missing, so the pilot cannot prove Postgres-to-Postgres convergence".to_string()
        },
        if local_stream && has_target {
            format!(
                "trellara run --local --verify --format text --config {config_path} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100"
            )
        } else {
            format!("trellara verify --config {config_path}")
        },
        "verify reports converged=true and checksum_status=match",
    )
}

pub(crate) fn failure_drill_gate(config_path: &str) -> PilotScorecardGate {
    PilotScorecardGate::new(
        "failure_drill",
        "repair and replay drill",
        PilotScorecardStatus::NeedsLiveEvidence,
        "quarantine and repair-plan commands expose the exact safe recovery action instead of generic lag",
        format!("trellara status --config {config_path} --view diagnostics --format text"),
        "a simulated target failure produces a replay-ready recovery plan without advancing target checkpoint incorrectly",
    )
}

pub(crate) fn lake_spark_consumption_gate(config_path: &str) -> PilotScorecardGate {
    PilotScorecardGate::new(
        "lake_spark_consumption",
        "lake fan-in and Spark consumption gate",
        PilotScorecardStatus::NeedsLiveEvidence,
        "raw CDC lake epochs require fan-in verification before Spark current-state, SCD2, or dashboard consumers are released",
        format!(
            "trellara lake fanin verify --config {config_path} --stream-epoch live-evidence/stream-epoch.json --lake-epoch live-evidence/lake-epoch.json --accept-complete-with-gaps --format text"
        ),
        "lake fan-in reports status=match, source_counts_match=true, checksum_rollup_match=true, and spark_consumption_allowed=true before derived Spark tables are trusted",
    )
}

pub(crate) fn lake_writer_plan_gate(config_path: &str) -> PilotScorecardGate {
    PilotScorecardGate::new(
        "lake_writer_plan",
        "raw CDC lake writer transaction and DDL boundary proof",
        PilotScorecardStatus::NeedsLiveEvidence,
        "lake writer row intents preserve source identity, transaction order, idempotent replay, and post-DDL schema metadata before Spark consumption is released",
        format!("trellara pilot-package --config {config_path}"),
        "lake-writer-plan.json includes row_intents, ordered commit steps, duplicate replay accounting, durability gates, and DDL boundary metadata",
    )
}

pub(crate) fn ddl_release_proof_gate(config_path: &str) -> PilotScorecardGate {
    PilotScorecardGate::new(
        "ddl_release_proof",
        "DDL barrier release proof",
        PilotScorecardStatus::NeedsLiveEvidence,
        "schema changes hold post-DDL DML until every required sink ACK proves it is ready at the same barrier boundary",
        format!("trellara pilot-package --config {config_path}"),
        "ddl-release-proof.json shows release_dml=true, post-DDL DML release, ACK commands, and required sink ACK evidence",
    )
}

pub(crate) fn strict_chunk_manifest_gate(config_path: &str) -> PilotScorecardGate {
    PilotScorecardGate::new(
        "strict_chunk_manifest",
        "strict chunk manifest and commit marker barrier",
        PilotScorecardStatus::ConfigurationReady,
        "strict_chunking is configured so oversized strict transactions are published as chunks plus manifest and commit marker barriers",
        format!("trellara contract-test --config {config_path}"),
        "oversized transactions become visible only after every chunk, the manifest, and the commit marker are durable",
    )
}

pub(crate) fn partition_watermarks_gate(config_path: &str) -> PilotScorecardGate {
    PilotScorecardGate::new(
        "partition_watermarks",
        "partitioned scale global visibility",
        PilotScorecardStatus::NeedsLiveEvidence,
        "partition manifests preserve transaction identity while partition watermarks define the global safe visibility point",
        format!("trellara partition-watermarks --config {config_path}"),
        "every partition has checkpoint evidence before global current-state visibility advances",
    )
}

pub(crate) fn partition_rebalance_plan_gate(config_path: &str) -> PilotScorecardGate {
    PilotScorecardGate::new(
        "partition_rebalance_plan",
        "partitioned scale rebalance governance",
        PilotScorecardStatus::NeedsLiveEvidence,
        "rebalance planning uses partition load plus checkpoint evidence, but runtime ownership movement remains disabled until reviewed cutover evidence exists",
        format!("trellara partition-rebalance-plan --config {config_path} --partition-event-count <partition>=<count> --format text"),
        "partition-rebalance-plan reports complete evidence, runtime_movement_allowed=false, and reviewable move candidates before any partition ownership change",
    )
}
