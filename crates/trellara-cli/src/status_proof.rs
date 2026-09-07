use serde::Serialize;

use crate::{
    ChecksumStatus, FlowStatusSummary, SnapshotHandoffProofStatus, TransactionBoundarySummary,
    ValidationProgress,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CorrectnessProofCheck {
    pub(crate) code: String,
    pub(crate) issue_code: Option<String>,
    pub(crate) status: CorrectnessProofStatus,
    pub(crate) evidence: String,
    pub(crate) recommendation: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CorrectnessProofStatus {
    Verified,
    AtRisk,
    MissingEvidence,
}

impl CorrectnessProofCheck {
    pub(crate) fn verified(code: impl Into<String>, evidence: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            issue_code: None,
            status: CorrectnessProofStatus::Verified,
            evidence: evidence.into(),
            recommendation: None,
        }
    }

    pub(crate) fn at_risk(
        code: impl Into<String>,
        evidence: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        let code = code.into();
        Self {
            issue_code: Some(format!("{code}_at_risk")),
            code,
            status: CorrectnessProofStatus::AtRisk,
            evidence: evidence.into(),
            recommendation: Some(recommendation.into()),
        }
    }

    pub(crate) fn missing_evidence(
        code: impl Into<String>,
        evidence: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        let code = code.into();
        Self {
            issue_code: Some(format!("{code}_missing_evidence")),
            code,
            status: CorrectnessProofStatus::MissingEvidence,
            evidence: evidence.into(),
            recommendation: Some(recommendation.into()),
        }
    }
}

pub(crate) struct CorrectnessProofInputs<'a> {
    pub(crate) status: &'a FlowStatusSummary,
    pub(crate) source_slot_safe: bool,
    pub(crate) source_subscription_conflicts_safe: bool,
    pub(crate) source_schema_contract_safe: bool,
    pub(crate) source_wal_retention_safe: bool,
    pub(crate) source_checkpoint_durable: bool,
    pub(crate) target_caught_up: bool,
    pub(crate) partition_watermark_ready: bool,
    pub(crate) no_target_quarantine: bool,
    pub(crate) snapshot_handoff_status: SnapshotHandoffProofStatus,
    pub(crate) latest_checksum_status: ChecksumStatus,
    pub(crate) validation_progress: ValidationProgress,
    pub(crate) transaction_boundary: &'a TransactionBoundarySummary,
}
