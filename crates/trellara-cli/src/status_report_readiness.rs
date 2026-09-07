use crate::{
    is_partitioned_mode, source_postgres_risk_factors, source_wal_retention_factor,
    subscription_conflict_factors, ChecksumStatus, FlowAlertSeverity, FlowStatusSummary,
    SnapshotHandoffProofStatus, TargetSourceProgress, ValidationProgress,
};

pub(crate) struct CorrectnessReportReadiness {
    pub(crate) ready: bool,
    pub(crate) source_slot_safe: bool,
    pub(crate) source_subscription_conflicts_safe: bool,
    pub(crate) source_wal_retention_safe: bool,
    pub(crate) source_checkpoint_durable: bool,
    pub(crate) target_caught_up: bool,
    pub(crate) partition_watermark_ready: bool,
    pub(crate) no_target_quarantine: bool,
    pub(crate) latest_validation_converged: bool,
    pub(crate) latest_validation_current: bool,
    pub(crate) latest_validation_source_lag_bytes: Option<u64>,
    pub(crate) latest_validation_target_lag_bytes: Option<u64>,
    pub(crate) latest_checksum_status: ChecksumStatus,
    pub(crate) snapshot_handoff_status: SnapshotHandoffProofStatus,
}

impl CorrectnessReportReadiness {
    pub(crate) fn from_status(status: &FlowStatusSummary) -> Self {
        let source_slot_safe = status.source_slot.exists
            && status.source_slot.issues.is_empty()
            && source_postgres_risk_factors(&status.source_slot).is_empty();
        let source_subscription_conflicts_safe =
            subscription_conflict_factors(&status.subscription_conflicts)
                .into_iter()
                .all(|factor| factor.severity != FlowAlertSeverity::Critical);
        let source_wal_retention_safe = source_wal_retention_factor(
            &status.source_slot,
            status.source_wal_retention_warn_bytes,
            "drain relay/apply lag or reseed slow targets before source WAL retention grows further",
        )
        .is_none();
        let source_checkpoint_durable = status
            .source
            .as_ref()
            .map(|source| source.source_is_durable)
            .unwrap_or(false);
        let target_caught_up = TargetSourceProgress::from_status(status).reaches_source_durable;
        let partition_watermark_ready = status
            .partition_watermarks
            .as_ref()
            .map(|watermarks| {
                watermarks.complete_partition_set
                    && watermarks
                        .global_durable_to_applied_bytes
                        .is_some_and(|lag| lag == 0)
            })
            .unwrap_or_else(|| !is_partitioned_mode(&status.mode));
        let no_target_quarantine = status.latest_quarantine.is_none();
        let latest_validation_converged = status
            .latest_validation
            .as_ref()
            .map(|validation| validation.converged)
            .unwrap_or(false);
        let latest_checksum_status = ChecksumStatus::from_validation(
            status
                .latest_validation
                .as_ref()
                .map(|validation| validation.converged),
        );
        let validation_progress = ValidationProgress::from_status(status);
        let snapshot_handoff_status = SnapshotHandoffProofStatus::from_status(status);
        let snapshot_handoff_ready = snapshot_handoff_status.is_verified();
        let ready = source_slot_safe
            && source_subscription_conflicts_safe
            && status.source_schema_drift.is_none()
            && source_checkpoint_durable
            && source_wal_retention_safe
            && target_caught_up
            && partition_watermark_ready
            && no_target_quarantine
            && snapshot_handoff_ready
            && latest_validation_converged
            && validation_progress.is_current;

        Self {
            ready,
            source_slot_safe,
            source_subscription_conflicts_safe,
            source_wal_retention_safe,
            source_checkpoint_durable,
            target_caught_up,
            partition_watermark_ready,
            no_target_quarantine,
            latest_validation_converged,
            latest_validation_current: validation_progress.is_current,
            latest_validation_source_lag_bytes: validation_progress.source_lag_bytes,
            latest_validation_target_lag_bytes: validation_progress.target_lag_bytes,
            latest_checksum_status,
            snapshot_handoff_status,
        }
    }
}
