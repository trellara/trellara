use crate::{DdlEvent, DdlPropagationDecision, ProtocolError};

pub(crate) fn validate_policy_release_gate(
    event: &DdlEvent,
    decision: &DdlPropagationDecision,
) -> Result<(), ProtocolError> {
    if decision.requires_target_ack && event.release_gate != decision.release_gate {
        return Err(ProtocolError::InvalidDdlEvent {
            total_order: event.total_order,
            reason: format!(
                "reviewable DDL must use release_gate={}",
                decision.release_gate
            ),
        });
    }
    if !decision.requires_target_ack && !event.release_gate.trim().is_empty() {
        return Err(ProtocolError::InvalidDdlEvent {
            total_order: event.total_order,
            reason: "unsupported DDL must not claim a release gate".to_string(),
        });
    }
    Ok(())
}
