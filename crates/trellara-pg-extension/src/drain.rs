use std::collections::HashSet;

use trellara_protocol::{parse_lsn, ProtocolError, TransactionBoundaryKey};

use crate::{NativeHandoffFrame, NativeHandoffQueueConfig};

#[path = "drain/types.rs"]
mod types;

pub use types::*;

pub fn native_handoff_drain_batch(
    config: &NativeHandoffQueueConfig,
    frames: Vec<NativeHandoffFrame>,
) -> Result<NativeHandoffDrainBatch, NativeHandoffDrainError> {
    if frames.is_empty() {
        return Err(NativeHandoffDrainError::EmptyBatch);
    }
    if frames.len() > config.capacity_frames {
        return Err(NativeHandoffDrainError::BatchExceedsQueueCapacity {
            frame_count: frames.len(),
            capacity_frames: config.capacity_frames,
        });
    }

    let payload_bytes = total_payload_bytes(&frames)?;
    if payload_bytes > config.max_buffered_payload_bytes {
        return Err(NativeHandoffDrainError::BatchPayloadExceedsQueueCapacity {
            payload_bytes,
            max_buffered_payload_bytes: config.max_buffered_payload_bytes,
        });
    }

    validate_drain_order(&frames)?;
    let first_commit_lsn = frames[0].commit_lsn.clone();
    let last_commit_lsn = frames[frames.len() - 1].commit_lsn.clone();

    Ok(NativeHandoffDrainBatch {
        contract: RELAY_HANDOFF_CONTRACT,
        frame_count: frames.len(),
        payload_bytes,
        first_commit_lsn,
        last_commit_lsn,
        frames,
        source_acknowledgement: crate::SOURCE_ACKNOWLEDGEMENT_CONTRACT,
    })
}

fn total_payload_bytes(frames: &[NativeHandoffFrame]) -> Result<usize, NativeHandoffDrainError> {
    frames.iter().try_fold(0usize, |total, frame| {
        total.checked_add(frame.payload_bytes).ok_or(
            NativeHandoffDrainError::BatchPayloadExceedsQueueCapacity {
                payload_bytes: usize::MAX,
                max_buffered_payload_bytes: usize::MAX,
            },
        )
    })
}

fn validate_drain_order(frames: &[NativeHandoffFrame]) -> Result<(), NativeHandoffDrainError> {
    let mut seen = HashSet::with_capacity(frames.len());
    let mut previous_lsn = None;
    for frame in frames {
        let boundary = TransactionBoundaryKey::new(
            frame.source_id.clone(),
            frame.dataset_id.clone(),
            frame.transaction_id.clone(),
            frame.commit_lsn.clone(),
        )
        .map_err(boundary_error)?;
        reject_duplicate_boundary(&mut seen, &boundary)?;

        let commit_lsn = parse_lsn(&boundary.commit_lsn).map_err(boundary_error)?;
        if let Some((previous_raw, previous)) = previous_lsn {
            if commit_lsn < previous {
                return Err(NativeHandoffDrainError::CommitLsnOrderRegression {
                    previous_commit_lsn: previous_raw,
                    current_commit_lsn: boundary.commit_lsn,
                });
            }
        }
        previous_lsn = Some((boundary.commit_lsn, commit_lsn));
    }
    Ok(())
}

fn reject_duplicate_boundary(
    seen: &mut HashSet<TransactionBoundaryKey>,
    boundary: &TransactionBoundaryKey,
) -> Result<(), NativeHandoffDrainError> {
    if seen.insert(boundary.clone()) {
        Ok(())
    } else {
        Err(NativeHandoffDrainError::DuplicateTransactionBoundary {
            boundary: boundary.to_string(),
        })
    }
}

fn boundary_error(error: ProtocolError) -> NativeHandoffDrainError {
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

fn invalid_boundary_field(
    field: &'static str,
    reason: impl Into<String>,
) -> NativeHandoffDrainError {
    NativeHandoffDrainError::InvalidBoundaryField {
        field,
        reason: reason.into(),
    }
}
