use sha2::{Digest, Sha256};

use crate::ddl_propagation_policy::ddl_policy_row;
use crate::ddl_propagation_validation::validate_policy_release_gate;
use crate::{DdlEvent, DdlOperation, ProtocolError, POST_DDL_DML_RELEASE_GATE};

pub const DDL_PROPAGATION_CDC_BOUNDARY: &str =
    "source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DdlPropagationDisposition {
    AutoApply,
    ManualReview,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DdlPropagationDecision {
    pub operation: DdlOperation,
    pub disposition: DdlPropagationDisposition,
    pub reason: &'static str,
    pub release_gate: &'static str,
    pub cdc_boundary: &'static str,
    pub requires_target_ack: bool,
}

impl DdlPropagationDecision {
    pub fn can_auto_apply(&self) -> bool {
        self.disposition == DdlPropagationDisposition::AutoApply
    }
}

pub fn classify_ddl_propagation(event: &DdlEvent) -> Result<DdlPropagationDecision, ProtocolError> {
    let operation = ddl_operation(event)?;
    let disposition = propagation_disposition(operation, event.target_auto_apply);
    Ok(DdlPropagationDecision {
        operation,
        disposition,
        reason: propagation_reason(operation, disposition),
        release_gate: POST_DDL_DML_RELEASE_GATE,
        cdc_boundary: DDL_PROPAGATION_CDC_BOUNDARY,
        requires_target_ack: requires_target_ack(disposition),
    })
}

pub fn default_target_auto_apply_for_ddl(operation: DdlOperation) -> bool {
    operation == DdlOperation::AddColumn
}

pub fn default_release_gate_for_ddl(operation: DdlOperation) -> &'static str {
    if supported_reviewable_ddl(operation) {
        POST_DDL_DML_RELEASE_GATE
    } else {
        ""
    }
}

pub fn ddl_propagation_policy_sha256(events: &[DdlEvent]) -> Result<String, ProtocolError> {
    let mut ordered_events = events.iter().collect::<Vec<_>>();
    ordered_events.sort_by_key(|event| event.total_order);
    let mut hasher = Sha256::new();
    hasher.update(DDL_PROPAGATION_CDC_BOUNDARY.as_bytes());
    hasher.update(b"\n");
    for event in ordered_events {
        let decision = classify_ddl_propagation(event)?;
        validate_policy_release_gate(event, &decision)?;
        hasher.update(ddl_policy_row(event, &decision).as_bytes());
        hasher.update(b"\n");
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn ddl_operation(event: &DdlEvent) -> Result<DdlOperation, ProtocolError> {
    DdlOperation::try_from(event.operation).map_err(|_| ProtocolError::UnsupportedDdlOperation {
        total_order: event.total_order,
        operation: event.operation,
    })
}

fn requires_target_ack(disposition: DdlPropagationDisposition) -> bool {
    matches!(
        disposition,
        DdlPropagationDisposition::AutoApply | DdlPropagationDisposition::ManualReview
    )
}

fn propagation_disposition(
    operation: DdlOperation,
    target_auto_apply: bool,
) -> DdlPropagationDisposition {
    match (operation, target_auto_apply) {
        (DdlOperation::AddColumn, true) => DdlPropagationDisposition::AutoApply,
        (DdlOperation::Unspecified | DdlOperation::Other, _) => {
            DdlPropagationDisposition::Unsupported
        }
        _ => DdlPropagationDisposition::ManualReview,
    }
}

fn supported_reviewable_ddl(operation: DdlOperation) -> bool {
    !matches!(operation, DdlOperation::Unspecified | DdlOperation::Other)
}

fn propagation_reason(
    operation: DdlOperation,
    disposition: DdlPropagationDisposition,
) -> &'static str {
    match (operation, disposition) {
        (DdlOperation::AddColumn, DdlPropagationDisposition::AutoApply) => {
            "additive ADD COLUMN can propagate inside the DDL barrier"
        }
        (DdlOperation::AddColumn, DdlPropagationDisposition::ManualReview) => {
            "requires manual review; additive ADD COLUMN is configured for operator review"
        }
        (_, DdlPropagationDisposition::ManualReview) => {
            "requires manual review; schema change requires operator approval before post-DDL DML release"
        }
        (_, DdlPropagationDisposition::Unsupported) => {
            "schema change is not safe to propagate automatically"
        }
        (_, DdlPropagationDisposition::AutoApply) => "schema change can auto-apply",
    }
}
