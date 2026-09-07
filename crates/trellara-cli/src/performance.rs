use std::path::Path;

use serde::Serialize;

use crate::{
    consistency_stream_kind, performance_capture_contract,
    performance_items::{performance_expected_bottlenecks, performance_tuning_levers},
    performance_proof_commands, performance_transaction_boundary_cost,
    performance_transport_durability_cost, TrellaraConfig, QUICKSTART_ESTIMATED_MINUTES,
    QUICKSTART_TIME_BUDGET_MINUTES,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PerformanceEnvelopeSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) mode: String,
    pub(crate) stream_kind: String,
    pub(crate) quickstart_estimated_minutes: u32,
    pub(crate) quickstart_time_budget_minutes: u32,
    pub(crate) default_relay_max_transactions: u64,
    pub(crate) default_apply_max_messages: u64,
    pub(crate) source_capture_contract: String,
    pub(crate) stream_spill_threshold_changes: usize,
    pub(crate) stream_spill_threshold_max_changes: usize,
    pub(crate) stream_spill_location: String,
    pub(crate) transaction_boundary_cost: String,
    pub(crate) transport_durability_cost: String,
    pub(crate) expected_bottlenecks: Vec<PerformanceEnvelopeItem>,
    pub(crate) tuning_levers: Vec<PerformanceEnvelopeItem>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) measurement_note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PerformanceEnvelopeItem {
    pub(crate) code: String,
    pub(crate) summary: String,
    pub(crate) evidence: String,
}

impl PerformanceEnvelopeSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Self {
        let config_path = path.display().to_string();
        let stream_spill_threshold_changes = config
            .source
            .stream_spill_threshold_changes
            .unwrap_or(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES);
        let stream_spill_location = config
            .source
            .stream_spill_dir
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "OS temp directory".to_string());

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            config: config_path.clone(),
            mode: config.status_mode(),
            stream_kind: consistency_stream_kind(&config.stream),
            quickstart_estimated_minutes: QUICKSTART_ESTIMATED_MINUTES,
            quickstart_time_budget_minutes: QUICKSTART_TIME_BUDGET_MINUTES,
            default_relay_max_transactions: 100,
            default_apply_max_messages: 100,
            source_capture_contract: performance_capture_contract(config),
            stream_spill_threshold_changes,
            stream_spill_threshold_max_changes:
                trellara_pg_capture::MAX_STREAM_SPILL_THRESHOLD_CHANGES,
            stream_spill_location,
            transaction_boundary_cost: performance_transaction_boundary_cost(config),
            transport_durability_cost: performance_transport_durability_cost(config),
            expected_bottlenecks: performance_expected_bottlenecks(config),
            tuning_levers: performance_tuning_levers(config),
            proof_commands: performance_proof_commands(config, &config_path),
            measurement_note:
                "This envelope is configuration-derived; live throughput claims require a captured run, stream inspection, status metrics, and verification evidence from the customer's workload."
                    .to_string(),
        }
    }
}

impl PerformanceEnvelopeItem {
    pub(crate) fn new(
        code: impl Into<String>,
        summary: impl Into<String>,
        evidence: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            summary: summary.into(),
            evidence: evidence.into(),
        }
    }
}
