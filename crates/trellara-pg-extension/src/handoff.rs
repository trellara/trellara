use serde::Serialize;
use trellara_protocol::{ProtocolError, TransactionBoundaryKey};

pub const MAX_HANDOFF_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum NativeHandoffFrameKind {
    CommittedTransaction,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeHandoffFrame {
    pub kind: NativeHandoffFrameKind,
    pub source_id: String,
    pub dataset_id: String,
    pub transaction_id: String,
    pub commit_lsn: String,
    pub payload_bytes: usize,
    pub envelope_checksum: u64,
    pub source_acknowledgement: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeHandoffError {
    InvalidBoundaryField {
        field: &'static str,
        reason: String,
    },
    EmptyPayload,
    PayloadTooLarge {
        payload_bytes: usize,
        max_payload_bytes: usize,
    },
    MissingEnvelopeChecksum,
}

pub fn native_committed_transaction_frame(
    source_id: &str,
    dataset_id: &str,
    transaction_id: &str,
    commit_lsn: &str,
    payload_bytes: usize,
    envelope_checksum: u64,
) -> Result<NativeHandoffFrame, NativeHandoffError> {
    let boundary = TransactionBoundaryKey::new(source_id, dataset_id, transaction_id, commit_lsn)
        .map_err(boundary_error)?;
    validate_payload(payload_bytes)?;
    validate_checksum(envelope_checksum)?;

    Ok(NativeHandoffFrame {
        kind: NativeHandoffFrameKind::CommittedTransaction,
        source_id: boundary.source_id,
        dataset_id: boundary.dataset_id,
        transaction_id: boundary.transaction_id,
        commit_lsn: boundary.commit_lsn,
        payload_bytes,
        envelope_checksum,
        source_acknowledgement: crate::SOURCE_ACKNOWLEDGEMENT_CONTRACT,
    })
}

fn validate_payload(payload_bytes: usize) -> Result<(), NativeHandoffError> {
    if payload_bytes == 0 {
        return Err(NativeHandoffError::EmptyPayload);
    }
    if payload_bytes > MAX_HANDOFF_PAYLOAD_BYTES {
        return Err(NativeHandoffError::PayloadTooLarge {
            payload_bytes,
            max_payload_bytes: MAX_HANDOFF_PAYLOAD_BYTES,
        });
    }
    Ok(())
}

fn validate_checksum(envelope_checksum: u64) -> Result<(), NativeHandoffError> {
    if envelope_checksum == 0 {
        Err(NativeHandoffError::MissingEnvelopeChecksum)
    } else {
        Ok(())
    }
}

fn boundary_error(error: ProtocolError) -> NativeHandoffError {
    match error {
        ProtocolError::MissingEnvelopeField { field } => {
            invalid_boundary_field(field, "must not be empty")
        }
        ProtocolError::InvalidEnvelopeField { field, reason } => {
            invalid_boundary_field(field, reason)
        }
        ProtocolError::InvalidLsn { reason, .. } => invalid_boundary_field("commit_lsn", reason),
        other => invalid_boundary_field("boundary", other.to_string()),
    }
}

fn invalid_boundary_field(field: &'static str, reason: impl Into<String>) -> NativeHandoffError {
    NativeHandoffError::InvalidBoundaryField {
        field,
        reason: reason.into(),
    }
}
