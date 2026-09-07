use serde::Serialize;

use crate::IdentityApplyGuarantee;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct IdentityAuditSummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) table_count: usize,
    pub(crate) pk_apply_ready_count: usize,
    pub(crate) live_identity_review_required_count: usize,
    pub(crate) cdc_apply_gate: String,
    pub(crate) toast_preservation_contract: String,
    pub(crate) ordinary_pk_tables_do_not_require_full: bool,
    pub(crate) apply_guarantees: Vec<IdentityApplyGuarantee>,
    pub(crate) tables: Vec<IdentityAuditTable>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) live_evidence_required: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct IdentityAuditTable {
    pub(crate) relation: String,
    pub(crate) configured_primary_key: Option<String>,
    pub(crate) apply_strategy: String,
    pub(crate) replica_identity_requirement: String,
    pub(crate) toast_handling: String,
    pub(crate) target_owned_columns: Vec<String>,
    pub(crate) status: IdentityAuditTableStatus,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IdentityAuditTableStatus {
    PkApplyReady,
    LiveIdentityReviewRequired,
}
