use serde::Serialize;
use trellara_checkpoint::{
    ApplyQuarantine, CheckpointLag, PartitionWatermarkSummary, ValidationEvent,
};
use trellara_pg_capture::ReplicationSlotStatus;

use crate::{FlowAlertSeverity, FlowSchemaDriftSummary};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowFailureSummary {
    pub(crate) code: String,
    pub(crate) severity: FlowAlertSeverity,
    pub(crate) message: String,
    pub(crate) occurred_at: Option<String>,
    pub(crate) recovery_action_codes: Vec<String>,
}

pub(crate) struct FlowFailureParts<'a> {
    pub(crate) source_slot: &'a ReplicationSlotStatus,
    pub(crate) source_wal_retention_warn_bytes: Option<i64>,
    pub(crate) source: Option<&'a CheckpointLag>,
    pub(crate) target: Option<&'a CheckpointLag>,
    pub(crate) partition_watermarks: Option<&'a PartitionWatermarkSummary>,
    pub(crate) source_schema_drift: Option<&'a FlowSchemaDriftSummary>,
    pub(crate) latest_quarantine: Option<&'a ApplyQuarantine>,
    pub(crate) latest_validation: Option<&'a ValidationEvent>,
    pub(crate) recovery_actions: &'a [FlowRecoveryAction],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowRecoveryAction {
    pub(crate) code: String,
    pub(crate) reason: String,
    pub(crate) transaction_id: Option<String>,
    pub(crate) commit_lsn: Option<String>,
    pub(crate) command_templates: Vec<String>,
    pub(crate) redelivery_topics: Vec<String>,
    pub(crate) redelivery_warnings: Vec<String>,
    pub(crate) hint: String,
}

pub(crate) fn validation_drift_message(validation: &ValidationEvent) -> String {
    let mut message = format!(
        "latest validation found drift in {} of {} tables at source LSN {}",
        validation.drift_count, validation.table_count, validation.source_watermark_lsn
    );
    if !validation.drift_relations.is_empty() {
        message.push_str("; relations: ");
        message.push_str(&validation.drift_relations.join(", "));
    }
    message
}
