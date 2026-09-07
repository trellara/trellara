use serde::Serialize;
use trellara_checkpoint::{
    ApplyQuarantine, CheckpointLag, PartitionWatermarkSummary, ReseedEvent, SnapshotHandoffEvent,
    SnapshotRun, ValidationEvent,
};
use trellara_pg_capture::{ReplicationSlotStatus, SubscriptionConflictStats};

use crate::{
    FlowFailureParts, FlowFailureSummary, FlowHealthParts, FlowHealthStatus, FlowHealthSummary,
    FlowRecoveryAction, FlowSchemaDriftSummary,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowStatusSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) health: FlowHealthSummary,
    pub(crate) source_checkpoint_exists: bool,
    pub(crate) target_checkpoint_exists: bool,
    pub(crate) source_slot: ReplicationSlotStatus,
    pub(crate) subscription_conflicts: Vec<SubscriptionConflictStats>,
    pub(crate) source_wal_retention_warn_bytes: Option<i64>,
    pub(crate) source_stream_spill_threshold_changes: usize,
    pub(crate) source_stream_spill_dir: Option<String>,
    pub(crate) source: Option<CheckpointLag>,
    pub(crate) target: Option<CheckpointLag>,
    pub(crate) partition_watermarks: Option<PartitionWatermarkSummary>,
    pub(crate) source_schema_drift: Option<FlowSchemaDriftSummary>,
    pub(crate) latest_quarantine: Option<ApplyQuarantine>,
    pub(crate) latest_reseed: Option<ReseedEvent>,
    pub(crate) latest_snapshot_handoff: Option<SnapshotHandoffEvent>,
    pub(crate) latest_snapshot_run: Option<SnapshotRun>,
    pub(crate) latest_validation: Option<ValidationEvent>,
    pub(crate) latest_failure: Option<FlowFailureSummary>,
    pub(crate) recovery_actions: Vec<FlowRecoveryAction>,
}

pub(crate) struct FlowStatusParts {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) source_slot: ReplicationSlotStatus,
    pub(crate) subscription_conflicts: Vec<SubscriptionConflictStats>,
    pub(crate) source_wal_retention_warn_bytes: Option<i64>,
    pub(crate) source_stream_spill_threshold_changes: usize,
    pub(crate) source_stream_spill_dir: Option<String>,
    pub(crate) source: Option<CheckpointLag>,
    pub(crate) target: Option<CheckpointLag>,
    pub(crate) partition_watermarks: Option<PartitionWatermarkSummary>,
    pub(crate) source_schema_drift: Option<FlowSchemaDriftSummary>,
    pub(crate) latest_quarantine: Option<ApplyQuarantine>,
    pub(crate) latest_reseed: Option<ReseedEvent>,
    pub(crate) latest_snapshot_handoff: Option<SnapshotHandoffEvent>,
    pub(crate) latest_snapshot_run: Option<SnapshotRun>,
    pub(crate) latest_validation: Option<ValidationEvent>,
    pub(crate) recovery_actions: Vec<FlowRecoveryAction>,
}

impl FlowStatusSummary {
    pub(crate) fn new(parts: FlowStatusParts) -> Self {
        let latest_failure = FlowFailureSummary::from_parts(FlowFailureParts {
            source_slot: &parts.source_slot,
            source_wal_retention_warn_bytes: parts.source_wal_retention_warn_bytes,
            source: parts.source.as_ref(),
            target: parts.target.as_ref(),
            partition_watermarks: parts.partition_watermarks.as_ref(),
            source_schema_drift: parts.source_schema_drift.as_ref(),
            latest_quarantine: parts.latest_quarantine.as_ref(),
            latest_validation: parts.latest_validation.as_ref(),
            recovery_actions: &parts.recovery_actions,
        });
        let mut health = FlowHealthSummary::from_parts(FlowHealthParts {
            source_slot: &parts.source_slot,
            subscription_conflicts: &parts.subscription_conflicts,
            source_wal_retention_warn_bytes: parts.source_wal_retention_warn_bytes,
            source: parts.source.as_ref(),
            target: parts.target.as_ref(),
            partition_watermarks: parts.partition_watermarks.as_ref(),
            latest_quarantine: parts.latest_quarantine.as_ref(),
            latest_validation: parts.latest_validation.as_ref(),
        });
        if let Some(drift) = parts.source_schema_drift.as_ref() {
            health.status = FlowHealthStatus::Blocked;
            health.issues.push(format!(
                "source schema drift requires fresh handoff for {}",
                drift.relations.join(", ")
            ));
            health.issue_count = health.issues.len();
        }
        Self {
            source_id: parts.source_id,
            dataset_id: parts.dataset_id,
            mode: parts.mode,
            health,
            source_checkpoint_exists: parts.source.is_some(),
            target_checkpoint_exists: parts.target.is_some(),
            source_slot: parts.source_slot,
            subscription_conflicts: parts.subscription_conflicts,
            source_wal_retention_warn_bytes: parts.source_wal_retention_warn_bytes,
            source_stream_spill_threshold_changes: parts.source_stream_spill_threshold_changes,
            source_stream_spill_dir: parts.source_stream_spill_dir,
            source: parts.source,
            target: parts.target,
            partition_watermarks: parts.partition_watermarks,
            source_schema_drift: parts.source_schema_drift,
            latest_quarantine: parts.latest_quarantine,
            latest_reseed: parts.latest_reseed,
            latest_snapshot_handoff: parts.latest_snapshot_handoff,
            latest_snapshot_run: parts.latest_snapshot_run,
            latest_validation: parts.latest_validation,
            latest_failure,
            recovery_actions: parts.recovery_actions,
        }
    }
}
