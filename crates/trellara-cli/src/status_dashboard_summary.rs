use crate::{
    CorrectnessProofStatus, CorrectnessReportSummary, DashboardSummary, FlowAlertsSummary,
    FlowStatusSummary, TargetSourceProgress, QUICKSTART_ESTIMATED_MINUTES,
    QUICKSTART_TIME_BUDGET_MINUTES,
};

impl DashboardSummary {
    pub(crate) fn from_status(status: FlowStatusSummary) -> Self {
        let report = CorrectnessReportSummary::from_status(status.clone());
        let alerts = FlowAlertsSummary::from_status(status.clone());
        let snapshot_handoff_check = report
            .proof_checks
            .iter()
            .find(|check| check.code == "snapshot_handoff");
        let snapshot_handoff_status = snapshot_handoff_check
            .map(|check| check.status)
            .unwrap_or(CorrectnessProofStatus::MissingEvidence);
        let snapshot_handoff_evidence = snapshot_handoff_check.map(|check| check.evidence.clone());
        let source_to_target_lag_bytes =
            TargetSourceProgress::from_status(&status).source_to_target_lag_bytes;

        Self {
            source_id: status.source_id,
            dataset_id: status.dataset_id,
            mode: status.mode,
            ready: report.ready,
            quickstart_estimated_minutes: QUICKSTART_ESTIMATED_MINUTES,
            quickstart_time_budget_minutes: QUICKSTART_TIME_BUDGET_MINUTES,
            status: status.health.status,
            issue_count: status.health.issue_count,
            proof_check_count: report.proof_check_count,
            at_risk_proof_check_count: report.at_risk_proof_check_count,
            missing_evidence_proof_check_count: report.missing_evidence_proof_check_count,
            proof_checks: report.proof_checks,
            transaction_boundary: report.transaction_boundary,
            latest_failure: status.latest_failure,
            source_watermark_lsn: report.latest_source_watermark_lsn,
            target_watermark_lsn: report.latest_target_watermark_lsn,
            source_to_target_lag_bytes,
            partition_global_applied_lsn: report.latest_partition_global_applied_lsn,
            latest_validation_converged: status
                .latest_validation
                .as_ref()
                .map(|validation| validation.converged),
            checksum_status: report.latest_checksum_status,
            snapshot_handoff_status,
            snapshot_handoff_evidence,
            latest_snapshot_handoff_relation: report.latest_snapshot_handoff_relation,
            latest_snapshot_state: report.latest_snapshot_state,
            latest_snapshot_consistent_lsn: report.latest_snapshot_consistent_lsn,
            recovery_actions: status.recovery_actions,
            alert_count: alerts.alert_count,
            highest_severity: alerts.highest_severity,
            alerts: alerts.alerts,
        }
    }
}
