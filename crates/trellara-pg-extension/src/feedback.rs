use serde::Serialize;
use trellara_protocol::parse_lsn;

use crate::NativeHandoffDrainBatch;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativePublishDestination {
    pub topic: String,
    pub partition: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeSourceFeedbackProof {
    pub durable_publish_lsn: String,
    pub published_frame_checksums: Vec<u64>,
    pub expected_publish_destinations: Vec<NativePublishDestination>,
    pub durable_publish_destinations: Vec<NativePublishDestination>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeSourceFeedbackDecision {
    pub source_feedback_lsn: String,
    pub frames_covered: usize,
    pub contract: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeSourceFeedbackError {
    DurableLsnBehindBatch {
        batch_last_commit_lsn: String,
        durable_publish_lsn: String,
    },
    FrameProofCountMismatch {
        expected_frames: usize,
        published_frames: usize,
    },
    FrameProofChecksumMismatch {
        frame_index: usize,
        expected_checksum: u64,
        published_checksum: u64,
    },
    PublishDestinationMismatch,
    InvalidDurablePublishLsn {
        reason: String,
    },
}

pub fn native_source_feedback_decision(
    batch: &NativeHandoffDrainBatch,
    proof: NativeSourceFeedbackProof,
) -> Result<NativeSourceFeedbackDecision, NativeSourceFeedbackError> {
    validate_frame_proofs(batch, &proof)?;
    validate_publish_destinations(&proof)?;
    validate_durable_lsn_covers_batch(batch, &proof.durable_publish_lsn)?;

    Ok(NativeSourceFeedbackDecision {
        source_feedback_lsn: batch.last_commit_lsn.clone(),
        frames_covered: batch.frame_count,
        contract: crate::SOURCE_ACKNOWLEDGEMENT_CONTRACT,
    })
}

fn validate_frame_proofs(
    batch: &NativeHandoffDrainBatch,
    proof: &NativeSourceFeedbackProof,
) -> Result<(), NativeSourceFeedbackError> {
    if proof.published_frame_checksums.len() != batch.frames.len() {
        return Err(NativeSourceFeedbackError::FrameProofCountMismatch {
            expected_frames: batch.frames.len(),
            published_frames: proof.published_frame_checksums.len(),
        });
    }

    for (index, (frame, published_checksum)) in batch
        .frames
        .iter()
        .zip(proof.published_frame_checksums.iter())
        .enumerate()
    {
        if frame.envelope_checksum != *published_checksum {
            return Err(NativeSourceFeedbackError::FrameProofChecksumMismatch {
                frame_index: index,
                expected_checksum: frame.envelope_checksum,
                published_checksum: *published_checksum,
            });
        }
    }
    Ok(())
}

fn validate_publish_destinations(
    proof: &NativeSourceFeedbackProof,
) -> Result<(), NativeSourceFeedbackError> {
    if proof.expected_publish_destinations == proof.durable_publish_destinations {
        Ok(())
    } else {
        Err(NativeSourceFeedbackError::PublishDestinationMismatch)
    }
}

fn validate_durable_lsn_covers_batch(
    batch: &NativeHandoffDrainBatch,
    durable_publish_lsn: &str,
) -> Result<(), NativeSourceFeedbackError> {
    let durable_lsn = parse_lsn(durable_publish_lsn).map_err(|error| {
        NativeSourceFeedbackError::InvalidDurablePublishLsn {
            reason: error.to_string(),
        }
    })?;
    let batch_lsn =
        parse_lsn(&batch.last_commit_lsn).expect("drain batch contains canonical commit LSNs");
    if durable_lsn < batch_lsn {
        return Err(NativeSourceFeedbackError::DurableLsnBehindBatch {
            batch_last_commit_lsn: batch.last_commit_lsn.clone(),
            durable_publish_lsn: durable_publish_lsn.to_owned(),
        });
    }
    Ok(())
}
