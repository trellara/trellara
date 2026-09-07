use serde::Serialize;

use crate::{
    ChecksumStatus, CorrectnessProofCheck, CorrectnessProofStatus, CorrectnessReportSummary,
    FlowAlert, FlowAlertSeverity, FlowAlertsSummary, FlowFailureSummary, FlowHealthStatus,
    FlowRecoveryAction, RepairPlanSummary, TransactionBoundarySummary,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DashboardSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) ready: bool,
    pub(crate) quickstart_estimated_minutes: u32,
    pub(crate) quickstart_time_budget_minutes: u32,
    pub(crate) status: FlowHealthStatus,
    pub(crate) issue_count: usize,
    pub(crate) proof_check_count: usize,
    pub(crate) at_risk_proof_check_count: usize,
    pub(crate) missing_evidence_proof_check_count: usize,
    pub(crate) proof_checks: Vec<CorrectnessProofCheck>,
    pub(crate) transaction_boundary: TransactionBoundarySummary,
    pub(crate) latest_failure: Option<FlowFailureSummary>,
    pub(crate) source_watermark_lsn: Option<String>,
    pub(crate) target_watermark_lsn: Option<String>,
    pub(crate) source_to_target_lag_bytes: Option<u64>,
    pub(crate) partition_global_applied_lsn: Option<String>,
    pub(crate) latest_validation_converged: Option<bool>,
    pub(crate) checksum_status: ChecksumStatus,
    pub(crate) snapshot_handoff_status: CorrectnessProofStatus,
    pub(crate) snapshot_handoff_evidence: Option<String>,
    pub(crate) latest_snapshot_handoff_relation: Option<String>,
    pub(crate) latest_snapshot_state: Option<String>,
    pub(crate) latest_snapshot_consistent_lsn: Option<String>,
    pub(crate) recovery_actions: Vec<FlowRecoveryAction>,
    pub(crate) alert_count: usize,
    pub(crate) highest_severity: Option<FlowAlertSeverity>,
    pub(crate) alerts: Vec<FlowAlert>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DiagnosticsBundleSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) config: String,
    pub(crate) ready: bool,
    pub(crate) status: FlowHealthStatus,
    pub(crate) latest_failure: Option<FlowFailureSummary>,
    pub(crate) report: CorrectnessReportSummary,
    pub(crate) alerts: FlowAlertsSummary,
    pub(crate) repair_plan: RepairPlanSummary,
    pub(crate) barrier_blockers: Vec<String>,
    pub(crate) metrics: String,
    pub(crate) attachment_commands: Vec<String>,
}
