use std::collections::HashSet;

use crate::{
    envelope_ddl_validation::validate_ddl_event, parse_lsn,
    schema_version_validation::validate_schema_versions, InvalidChangeFieldError, ProtocolError,
    TransactionEnvelope, PROTOCOL_VERSION,
};

pub(crate) fn validate_envelope(envelope: &TransactionEnvelope) -> Result<(), ProtocolError> {
    if envelope.protocol_version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedProtocolVersion {
            actual: envelope.protocol_version,
            expected: PROTOCOL_VERSION,
        });
    }
    require_field("source_id", &envelope.source_id)?;
    require_field("database_id", &envelope.database_id)?;
    require_field("dataset_id", &envelope.dataset_id)?;
    require_field("transaction_id", &envelope.transaction_id)?;
    require_field("commit_lsn", &envelope.commit_lsn)?;
    if envelope.commit_timestamp_ms <= 0 {
        return Err(ProtocolError::InvalidEnvelopeField {
            field: "commit_timestamp_ms",
            reason: "must be greater than zero".to_string(),
        });
    }

    let commit_lsn = parse_nonzero_lsn(&envelope.commit_lsn)?;
    if !envelope.begin_lsn.trim().is_empty() {
        let begin_lsn = parse_nonzero_lsn(&envelope.begin_lsn)?;
        if begin_lsn > commit_lsn {
            return Err(ProtocolError::EnvelopeLsnOrder {
                begin_lsn: envelope.begin_lsn.clone(),
                commit_lsn: envelope.commit_lsn.clone(),
            });
        }
    }

    validate_transaction_events(envelope)?;
    validate_schema_versions(&envelope.schema_versions)
}

fn validate_transaction_events(envelope: &TransactionEnvelope) -> Result<(), ProtocolError> {
    let mut event_orders =
        HashSet::with_capacity(envelope.changes.len() + envelope.ddl_events.len());
    for change in &envelope.changes {
        if change.transaction_id != envelope.transaction_id {
            return Err(ProtocolError::ChangeTransactionMismatch {
                total_order: change.total_order,
                expected: envelope.transaction_id.clone(),
                actual: change.transaction_id.clone(),
            });
        }
        validate_change_idempotency_key(change.total_order, &change.idempotency_key)?;
        insert_event_order(&mut event_orders, change.total_order)?;
    }
    for event in &envelope.ddl_events {
        validate_ddl_event(&envelope.transaction_id, event)?;
        insert_event_order(&mut event_orders, event.total_order)?;
    }
    Ok(())
}

fn validate_change_idempotency_key(
    total_order: u32,
    idempotency_key: &str,
) -> Result<(), ProtocolError> {
    if idempotency_key.trim().is_empty() {
        return Err(InvalidChangeFieldError {
            total_order,
            field: "idempotency_key",
            reason: "must not be empty".to_string(),
        }
        .into());
    }
    if idempotency_key.trim() != idempotency_key {
        return Err(InvalidChangeFieldError {
            total_order,
            field: "idempotency_key",
            reason: "must not contain surrounding whitespace".to_string(),
        }
        .into());
    }
    Ok(())
}

fn insert_event_order(
    event_orders: &mut HashSet<u32>,
    total_order: u32,
) -> Result<(), ProtocolError> {
    if !event_orders.insert(total_order) {
        return Err(ProtocolError::DuplicateTransactionEventOrder { total_order });
    }
    Ok(())
}

fn require_field(field: &'static str, value: &str) -> Result<(), ProtocolError> {
    if value.trim().is_empty() {
        Err(ProtocolError::MissingEnvelopeField { field })
    } else if value.trim() != value {
        Err(ProtocolError::InvalidEnvelopeField {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        })
    } else {
        Ok(())
    }
}

fn parse_nonzero_lsn(lsn: &str) -> Result<u64, ProtocolError> {
    let parsed = parse_lsn(lsn)?;
    if parsed == 0 {
        return Err(ProtocolError::InvalidLsn {
            lsn: lsn.to_string(),
            reason: "LSN must be greater than zero".to_string(),
        });
    }
    Ok(parsed)
}
