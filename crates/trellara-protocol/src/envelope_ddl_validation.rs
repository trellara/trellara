use crate::{
    classify_ddl_propagation, DdlEvent, DdlOperation, DdlPropagationDisposition, ProtocolError,
    POST_DDL_DML_RELEASE_GATE,
};

pub(crate) fn validate_ddl_event(
    transaction_id: &str,
    event: &DdlEvent,
) -> Result<(), ProtocolError> {
    if event.transaction_id != transaction_id {
        return Err(ProtocolError::DdlTransactionMismatch {
            total_order: event.total_order,
            expected: transaction_id.to_string(),
            actual: event.transaction_id.clone(),
        });
    }
    if event.total_order == 0 {
        return Err(ProtocolError::InvalidDdlEvent {
            total_order: event.total_order,
            reason: "total_order must be greater than zero".to_string(),
        });
    }
    if event.relation.is_none() {
        return Err(ProtocolError::MissingDdlEventField {
            total_order: event.total_order,
            field: "relation",
        });
    }
    require_ddl_field(event.total_order, "statement", &event.statement)?;
    let decision = classify_ddl_propagation(event)?;
    let operation = decision.operation;
    if operation == DdlOperation::Unspecified {
        return Err(ProtocolError::InvalidDdlEvent {
            total_order: event.total_order,
            reason: "operation must be specified".to_string(),
        });
    }
    if event.schema_fingerprint_after == 0 {
        return Err(ProtocolError::InvalidDdlEvent {
            total_order: event.total_order,
            reason: "schema_fingerprint_after must be greater than zero".to_string(),
        });
    }
    if event.schema_fingerprint_before == event.schema_fingerprint_after {
        return Err(ProtocolError::InvalidDdlEvent {
            total_order: event.total_order,
            reason: "schema fingerprints must change across DDL".to_string(),
        });
    }
    if event.target_auto_apply {
        if decision.disposition != DdlPropagationDisposition::AutoApply {
            return Err(ProtocolError::InvalidDdlEvent {
                total_order: event.total_order,
                reason: "target_auto_apply DDL must use operation=add_column".to_string(),
            });
        }
        require_ddl_field(event.total_order, "release_gate", &event.release_gate)?;
        if event.release_gate != POST_DDL_DML_RELEASE_GATE {
            return Err(ProtocolError::InvalidDdlEvent {
                total_order: event.total_order,
                reason: format!(
                    "target_auto_apply DDL must use release_gate={POST_DDL_DML_RELEASE_GATE}"
                ),
            });
        }
    }
    if decision.requires_target_ack {
        require_ddl_field(event.total_order, "release_gate", &event.release_gate)?;
        if event.release_gate != decision.release_gate {
            return Err(ProtocolError::InvalidDdlEvent {
                total_order: event.total_order,
                reason: format!(
                    "reviewable DDL must use release_gate={}",
                    decision.release_gate
                ),
            });
        }
    }
    Ok(())
}

fn require_ddl_field(
    total_order: u32,
    field: &'static str,
    value: &str,
) -> Result<(), ProtocolError> {
    if value.trim().is_empty() {
        Err(ProtocolError::MissingDdlEventField { total_order, field })
    } else {
        Ok(())
    }
}
