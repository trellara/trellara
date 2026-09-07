use std::path::Path;

use crate::{
    pilot_acceptance_gates::pilot_acceptance_gates, pilot_evidence_commands, pilot_failure_drill,
    pilot_guide_phases, pilot_large_transaction_evidence, quickstart_capture_spill_message,
    PilotGuidePhase, PilotGuideSummary, StreamConfig, TrellaraConfig,
    QUICKSTART_TIME_BUDGET_MINUTES,
};

impl PilotGuideSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Self {
        let config_path = path.display().to_string();
        let stream_kind = match &config.stream {
            StreamConfig::Kafka { .. } => "kafka".to_string(),
            StreamConfig::Local { .. } => "local".to_string(),
        };
        let kafka_required = !matches!(config.stream, StreamConfig::Local { .. });
        let phases = pilot_guide_phases(config, &config_path);
        let evidence_commands = pilot_evidence_commands(config, &config_path);
        let large_transaction_evidence = pilot_large_transaction_evidence(config, &config_path);
        let failure_drill = pilot_failure_drill(config, &config_path);
        let acceptance_gates = pilot_acceptance_gates(config);

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            config: config_path,
            objective: "evaluate one verified Postgres replication flow with source-safety, snapshot handoff, failure drill, and proof artifacts".to_string(),
            evaluation_time_budget_minutes: QUICKSTART_TIME_BUDGET_MINUTES,
            kafka_required,
            stream_kind,
            transaction_boundary: config.explain(),
            capture_spill_boundary: quickstart_capture_spill_message(config),
            table_count: config.dataset.tables.len(),
            tables: config
                .dataset
                .tables
                .iter()
                .map(|table| table.relation_id().display_name())
                .collect(),
            phase_count: phases.len(),
            phases,
            evidence_commands,
            large_transaction_evidence,
            failure_drill,
            acceptance_gates,
        }
    }
}

impl PilotGuidePhase {
    pub(crate) fn new(
        order: usize,
        name: impl Into<String>,
        command: impl Into<String>,
        proof: impl Into<String>,
    ) -> Self {
        Self {
            order,
            name: name.into(),
            command: command.into(),
            proof: proof.into(),
        }
    }
}
