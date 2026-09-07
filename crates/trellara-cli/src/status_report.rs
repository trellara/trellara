use serde::Serialize;

use crate::{
    correctness_proof_checks, report_recommended_actions, ChecksumStatus, CorrectnessProofCheck,
    CorrectnessProofInputs, CorrectnessReportReadiness, FlowFailureSummary, FlowStatusSummary,
    ReportActionInputs, TargetSourceProgress, TransactionBoundarySummary,
};
use crate::{
    status_report_proof_counts::CorrectnessProofCounts,
    status_report_watermarks::CorrectnessReportWatermarks,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CorrectnessReportSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) ready: bool,
    pub(crate) proof_check_count: usize,
    pub(crate) verified_proof_check_count: usize,
    pub(crate) at_risk_proof_check_count: usize,
    pub(crate) missing_evidence_proof_check_count: usize,
    pub(crate) proof_checks: Vec<CorrectnessProofCheck>,
    pub(crate) transaction_boundary: TransactionBoundarySummary,
    pub(crate) source_slot_safe: bool,
    pub(crate) source_subscription_conflicts_safe: bool,
    pub(crate) source_schema_contract_safe: bool,
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
    pub(crate) latest_source_watermark_lsn: Option<String>,
    pub(crate) latest_target_watermark_lsn: Option<String>,
    pub(crate) latest_partition_global_applied_lsn: Option<String>,
    pub(crate) latest_reseed_watermark_lsn: Option<String>,
    pub(crate) latest_snapshot_handoff_watermark_lsn: Option<String>,
    pub(crate) latest_snapshot_handoff_relation: Option<String>,
    pub(crate) latest_snapshot_run_id: Option<String>,
    pub(crate) latest_snapshot_state: Option<String>,
    pub(crate) latest_snapshot_consistent_lsn: Option<String>,
    pub(crate) latest_failure: Option<FlowFailureSummary>,
    pub(crate) issues: Vec<String>,
    pub(crate) recommended_actions: Vec<String>,
}

impl CorrectnessReportSummary {
    pub(crate) fn from_status(status: FlowStatusSummary) -> Self {
        let transaction_boundary = TransactionBoundarySummary::from_status(&status);
        let readiness = CorrectnessReportReadiness::from_status(&status);
        let source_schema_contract_safe = status.source_schema_drift.is_none();
        let proof_checks = correctness_proof_checks(CorrectnessProofInputs {
            status: &status,
            source_slot_safe: readiness.source_slot_safe,
            source_subscription_conflicts_safe: readiness.source_subscription_conflicts_safe,
            source_schema_contract_safe,
            source_wal_retention_safe: readiness.source_wal_retention_safe,
            source_checkpoint_durable: readiness.source_checkpoint_durable,
            target_caught_up: readiness.target_caught_up,
            partition_watermark_ready: readiness.partition_watermark_ready,
            no_target_quarantine: readiness.no_target_quarantine,
            snapshot_handoff_status: readiness.snapshot_handoff_status,
            latest_checksum_status: readiness.latest_checksum_status,
            validation_progress: crate::ValidationProgress {
                is_current: readiness.latest_validation_current,
                source_lag_bytes: readiness.latest_validation_source_lag_bytes,
                target_lag_bytes: readiness.latest_validation_target_lag_bytes,
            },
            transaction_boundary: &transaction_boundary,
        });
        let proof_counts = CorrectnessProofCounts::from_checks(&proof_checks);
        let recommended_actions = report_recommended_actions(ReportActionInputs {
            source_slot: &status.source_slot,
            source_subscription_conflicts_safe: readiness.source_subscription_conflicts_safe,
            source_schema_contract_safe,
            source_schema_drift: status.source_schema_drift.as_ref(),
            source_wal_retention_safe: readiness.source_wal_retention_safe,
            source_checkpoint_durable: readiness.source_checkpoint_durable,
            target_caught_up: readiness.target_caught_up,
            source_to_target_lag_bytes: TargetSourceProgress::from_status(&status)
                .source_to_target_lag_bytes,
            partition_watermark_ready: readiness.partition_watermark_ready,
            no_target_quarantine: readiness.no_target_quarantine,
            snapshot_handoff_status: readiness.snapshot_handoff_status,
            latest_validation_converged: readiness.latest_validation_converged,
            latest_validation_current: readiness.latest_validation_current,
        });
        let watermarks = CorrectnessReportWatermarks::from_status(&status);

        Self {
            source_id: status.source_id,
            dataset_id: status.dataset_id,
            mode: status.mode,
            ready: readiness.ready,
            proof_check_count: proof_counts.total,
            verified_proof_check_count: proof_counts.verified,
            at_risk_proof_check_count: proof_counts.at_risk,
            missing_evidence_proof_check_count: proof_counts.missing_evidence,
            proof_checks,
            transaction_boundary,
            source_slot_safe: readiness.source_slot_safe,
            source_subscription_conflicts_safe: readiness.source_subscription_conflicts_safe,
            source_schema_contract_safe,
            source_wal_retention_safe: readiness.source_wal_retention_safe,
            source_checkpoint_durable: readiness.source_checkpoint_durable,
            target_caught_up: readiness.target_caught_up,
            partition_watermark_ready: readiness.partition_watermark_ready,
            no_target_quarantine: readiness.no_target_quarantine,
            latest_validation_converged: readiness.latest_validation_converged,
            latest_validation_current: readiness.latest_validation_current,
            latest_validation_source_lag_bytes: readiness.latest_validation_source_lag_bytes,
            latest_validation_target_lag_bytes: readiness.latest_validation_target_lag_bytes,
            latest_checksum_status: readiness.latest_checksum_status,
            latest_source_watermark_lsn: watermarks.source_lsn,
            latest_target_watermark_lsn: watermarks.target_lsn,
            latest_partition_global_applied_lsn: watermarks.partition_global_applied_lsn,
            latest_reseed_watermark_lsn: watermarks.reseed_lsn,
            latest_snapshot_handoff_watermark_lsn: watermarks.snapshot_handoff_watermark_lsn,
            latest_snapshot_handoff_relation: watermarks.snapshot_handoff_relation,
            latest_snapshot_run_id: watermarks.snapshot_run_id,
            latest_snapshot_state: watermarks.snapshot_state,
            latest_snapshot_consistent_lsn: watermarks.snapshot_consistent_lsn,
            latest_failure: status.latest_failure.clone(),
            issues: status.health.issues,
            recommended_actions,
        }
    }
}
