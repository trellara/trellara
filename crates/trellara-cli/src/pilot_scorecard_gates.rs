use crate::{
    brokerless_evaluation_gate, ddl_release_proof_gate, failure_drill_gate,
    lake_spark_consumption_gate, lake_writer_plan_gate, partition_rebalance_plan_gate,
    partition_watermarks_gate, quickstart_capture_spill_message, strict_chunk_manifest_gate,
    verified_apply_gate, DatasetMode, PilotScorecardGate, PilotScorecardStatus, TrellaraConfig,
};

pub(crate) fn pilot_scorecard_gates(
    config: &TrellaraConfig,
    config_path: &str,
    local_stream: bool,
    has_target: bool,
) -> Vec<PilotScorecardGate> {
    let mut gates = core_pilot_scorecard_gates(config, config_path);
    gates.push(brokerless_evaluation_gate(config_path, local_stream));
    gates.push(verified_apply_gate(config_path, local_stream, has_target));
    gates.push(ddl_release_proof_gate(config_path));
    gates.push(lake_writer_plan_gate(config_path));
    gates.push(lake_spark_consumption_gate(config_path));
    gates.push(failure_drill_gate(config_path));

    if config.dataset.strict_chunking.is_some() {
        gates.push(strict_chunk_manifest_gate(config_path));
    }
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        gates.push(partition_watermarks_gate(config_path));
        gates.push(partition_rebalance_plan_gate(config_path));
    }

    gates
}

fn core_pilot_scorecard_gates(
    config: &TrellaraConfig,
    config_path: &str,
) -> Vec<PilotScorecardGate> {
    vec![
        PilotScorecardGate::new(
            "source_safety",
            "read-only CDC source-safety assessment",
            PilotScorecardStatus::NeedsLiveEvidence,
            "slot posture, WAL retention, failover slot state, table identity, and subscriber conflict risk are evaluated without mutating the source",
            format!("trellara check --config {config_path} --format text"),
            "no critical slot, WAL-retention, or table-identity blockers remain before CDC starts",
        ),
        PilotScorecardGate::new(
            "contract_preflight",
            "source and target contract preflight",
            PilotScorecardStatus::NeedsLiveEvidence,
            "configured tables, schema fingerprints, target compatibility, and transaction-boundary mode are checked before capture",
            format!("trellara contract-test --config {config_path}"),
            "contract-test returns ready=true or only accepted advisory notes",
        ),
        PilotScorecardGate::new(
            "transaction_boundary",
            "transaction boundary is proven from live checkpoints",
            PilotScorecardStatus::NeedsLiveEvidence,
            config.explain(),
            format!("trellara status --config {config_path} --view report --format text"),
            "status report marks the transaction boundary verified with source and target checkpoint evidence before the target is trusted",
        ),
        PilotScorecardGate::new(
            "bounded_memory_capture",
            "large transactions have a bounded-memory capture path",
            PilotScorecardStatus::ConfigurationReady,
            quickstart_capture_spill_message(config),
            format!("trellara quickstart --config {config_path} --check"),
            "pgoutput streamed changes spill before commit; pilot run proof must include bounded_memory_contract and visibility_contract evidence for the configured manifest barrier",
        ),
        PilotScorecardGate::new(
            "snapshot_handoff",
            "initial snapshot-to-stream handoff",
            PilotScorecardStatus::NeedsLiveEvidence,
            "snapshot run records slot, consistent LSN, table progress, copied row counts, and handoff watermark",
            format!("trellara snapshot --config {config_path} --run-id pilot-snapshot-1"),
            "every selected table reaches stream_handoff_ready before stream replay is considered complete",
        ),
    ]
}
