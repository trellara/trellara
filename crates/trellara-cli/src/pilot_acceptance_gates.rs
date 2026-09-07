use crate::{DatasetMode, StreamConfig, TrellaraConfig, QUICKSTART_TIME_BUDGET_MINUTES};

pub(crate) fn pilot_acceptance_gates(config: &TrellaraConfig) -> Vec<String> {
    let mut acceptance_gates = vec![
        format!(
            "first verified flow completes in {} minutes or less",
            QUICKSTART_TIME_BUDGET_MINUTES
        ),
        "source-safety returns no critical source slot or replica identity blockers".to_string(),
        "snapshot reaches stream_handoff_ready for every selected table".to_string(),
        "verify reports converged=true and checksum_status=match".to_string(),
        "status --view report returns ready=true with verified transaction-boundary proof"
            .to_string(),
        "quarantine replay-ready provides exact redelivery topics if a target repair is needed"
            .to_string(),
    ];
    if matches!(config.stream, StreamConfig::Local { .. }) {
        acceptance_gates.push("pilot can run without adopting Kafka or a managed broker, and stream inspect-local reports clean or recovered local stream health".to_string());
    }
    acceptance_gates.push(
        "large transactions stay bounded by pgoutput spill and preserve commit visibility only at the configured transaction boundary"
            .to_string(),
    );
    if config.dataset.strict_chunking.is_some() {
        acceptance_gates.push(
            "strict chunk manifests make oversized transactions visible only after every chunk, the manifest, and the commit marker are durable"
                .to_string(),
        );
    }
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        acceptance_gates.push(
            "partition-watermarks reports every partition complete before global visibility"
                .to_string(),
        );
    }

    acceptance_gates
}
