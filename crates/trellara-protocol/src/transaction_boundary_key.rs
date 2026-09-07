use std::fmt;

use crate::{format_lsn, idempotency_key, parse_lsn, ProtocolError, TransactionEnvelope};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransactionBoundaryKey {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub transaction_id: String,
    pub commit_lsn: String,
}

impl TransactionBoundaryKey {
    pub fn new(
        source_id: impl Into<String>,
        dataset_id: impl Into<String>,
        transaction_id: impl Into<String>,
        commit_lsn: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        Self::new_with_database(source_id, "", dataset_id, transaction_id, commit_lsn)
    }

    pub fn new_with_database(
        source_id: impl Into<String>,
        database_id: impl Into<String>,
        dataset_id: impl Into<String>,
        transaction_id: impl Into<String>,
        commit_lsn: impl Into<String>,
    ) -> Result<Self, ProtocolError> {
        let source_id = required_string("source_id", source_id.into())?;
        let database_id = optional_clean_string("database_id", database_id.into())?;
        let dataset_id = required_string("dataset_id", dataset_id.into())?;
        let transaction_id = required_string("transaction_id", transaction_id.into())?;
        let commit_lsn = canonical_nonzero_lsn(commit_lsn.into())?;
        Ok(Self {
            source_id,
            database_id,
            dataset_id,
            transaction_id,
            commit_lsn,
        })
    }

    pub fn from_envelope(envelope: &TransactionEnvelope) -> Result<Self, ProtocolError> {
        Self::new_with_database(
            envelope.source_id.clone(),
            envelope.database_id.clone(),
            envelope.dataset_id.clone(),
            envelope.transaction_id.clone(),
            envelope.commit_lsn.clone(),
        )
    }

    pub fn event_idempotency_key(&self, total_order: u32) -> String {
        idempotency_key(
            &self.source_id,
            &self.commit_lsn,
            &self.transaction_id,
            total_order,
        )
    }
}

impl fmt::Display for TransactionBoundaryKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.database_id.is_empty() {
            write!(
                formatter,
                "{}:{}:{}:{}",
                self.source_id, self.dataset_id, self.transaction_id, self.commit_lsn
            )
        } else {
            write!(
                formatter,
                "{}:{}:{}:{}:{}",
                self.source_id,
                self.database_id,
                self.dataset_id,
                self.transaction_id,
                self.commit_lsn
            )
        }
    }
}

fn required_string(field: &'static str, value: String) -> Result<String, ProtocolError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(ProtocolError::MissingEnvelopeField { field })
    } else if trimmed != value {
        Err(ProtocolError::InvalidEnvelopeField {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        })
    } else {
        Ok(value)
    }
}

fn optional_clean_string(field: &'static str, value: String) -> Result<String, ProtocolError> {
    if value.trim() != value {
        Err(ProtocolError::InvalidEnvelopeField {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        })
    } else {
        Ok(value)
    }
}

fn canonical_nonzero_lsn(value: String) -> Result<String, ProtocolError> {
    let trimmed = required_string("commit_lsn", value)?;
    let parsed = parse_lsn(&trimmed)?;
    if parsed == 0 {
        return Err(ProtocolError::InvalidLsn {
            lsn: trimmed,
            reason: "LSN must be greater than zero".to_string(),
        });
    }
    Ok(format_lsn(parsed))
}
