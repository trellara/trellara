use serde::Serialize;

use crate::{enterprise_run_proof_command, DatasetMode, StreamConfig, TrellaraConfig};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct EnterpriseReadinessGate {
    pub(crate) area: String,
    pub(crate) artifact: String,
    pub(crate) command: String,
    pub(crate) pass_condition: String,
}

pub(crate) fn enterprise_readiness_gates(
    config: &TrellaraConfig,
    config_path: &str,
    local_stream: bool,
) -> Vec<EnterpriseReadinessGate> {
    let mut gates = vec![
        source_safety_gate(config_path),
        ddl_governance_gate(config_path),
        recovery_posture_gate(config_path),
    ];
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        gates.push(partition_scale_gate(config_path));
    }
    if matches!(config.stream, StreamConfig::Local { .. }) {
        gates.push(local_durability_gate(config_path));
    }
    gates.push(lake_fanin_gate(config_path, local_stream));
    gates
}

fn source_safety_gate(config_path: &str) -> EnterpriseReadinessGate {
    EnterpriseReadinessGate {
        area: "source_safety".to_string(),
        artifact: "source-safety.txt".to_string(),
        command: format!("trellara check --config {config_path} --format text"),
        pass_condition:
            "no critical slot, WAL retention, replica identity, failover slot, or subscription conflict blockers"
                .to_string(),
    }
}

fn ddl_governance_gate(config_path: &str) -> EnterpriseReadinessGate {
    EnterpriseReadinessGate {
        area: "ddl_governance".to_string(),
        artifact: "schema-ddl-plan.json".to_string(),
        command: format!(
            "trellara schema ddl-plan --config {config_path} --format json --apply-mode manual-review --change <ddl-kind:relation>"
        ),
        pass_condition:
            "DDL propagation plan names active policy modes, release gates, required sink ACKs, and any blocked or manual-review changes"
                .to_string(),
    }
}

fn recovery_posture_gate(config_path: &str) -> EnterpriseReadinessGate {
    EnterpriseReadinessGate {
        area: "recovery_posture".to_string(),
        artifact: "diagnostics.txt + local-stream-inspection.json".to_string(),
        command: format!(
            "trellara status --config {config_path} --view diagnostics --format text && trellara stream inspect-local --config {config_path}"
        ),
        pass_condition:
            "diagnostics include latest failure, quarantine, repair-plan, replay-ready command, proof-check status, and stream recovery.recovery_ready with no torn-tail or cursor blockers"
                .to_string(),
    }
}

fn partition_scale_gate(config_path: &str) -> EnterpriseReadinessGate {
    EnterpriseReadinessGate {
        area: "partition_scale".to_string(),
        artifact: "partition-watermarks.json".to_string(),
        command: format!("trellara partition-watermarks --config {config_path} --format json"),
        pass_condition:
            "complete partition set with no lagging, straggler, blocking, or missing partitions before global visibility release"
                .to_string(),
    }
}

fn local_durability_gate(config_path: &str) -> EnterpriseReadinessGate {
    EnterpriseReadinessGate {
        area: "local_durability".to_string(),
        artifact: "local-stream-inspection.txt".to_string(),
        command: format!("trellara stream inspect-local --config {config_path}"),
        pass_condition:
            "local segment inspection reports replayable durable offsets, recovery.recovery_ready, healthy or rebuilt indexes, barrier topics, and no torn tail"
                .to_string(),
    }
}

fn lake_fanin_gate(config_path: &str, local_stream: bool) -> EnterpriseReadinessGate {
    EnterpriseReadinessGate {
        area: "lake_fanin".to_string(),
        artifact: "lake-verify.json".to_string(),
        command: format!(
            "{} && trellara lake fanin verify --config {config_path} --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json> --format json",
            enterprise_run_proof_command(config_path, local_stream)
        ),
        pass_condition:
            "lake verification status is match, source_counts_match=true, checksum_rollup_match=true, manifest digest recomputes from epoch rows, and Spark consumption is explicitly gated"
                .to_string(),
    }
}
