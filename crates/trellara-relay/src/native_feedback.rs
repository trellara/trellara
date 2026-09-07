use trellara_pg_extension::{
    native_source_feedback_decision, NativeHandoffDrainBatch, NativePublishDestination,
    NativeSourceFeedbackDecision, NativeSourceFeedbackProof,
};

use crate::{PublishDestination, RelayError, RelayStep, Result};

pub fn native_feedback_proof_from_relay_step(
    step: &RelayStep,
    published_frame_checksums: Vec<u64>,
) -> Result<NativeSourceFeedbackProof> {
    validate_native_feedback_ready(step)?;
    Ok(NativeSourceFeedbackProof {
        durable_publish_lsn: step.source_ack_boundary.source_ack_lsn.clone(),
        published_frame_checksums,
        expected_publish_destinations: native_destinations(
            &step.source_ack_boundary.expected_publish_destinations,
        ),
        durable_publish_destinations: native_destinations(
            &step.source_ack_boundary.durable_publish_destinations,
        ),
    })
}

pub fn native_feedback_decision_from_relay_step(
    batch: &NativeHandoffDrainBatch,
    step: &RelayStep,
) -> Result<NativeSourceFeedbackDecision> {
    let checksums = batch
        .frames
        .iter()
        .map(|frame| frame.envelope_checksum)
        .collect();
    let proof = native_feedback_proof_from_relay_step(step, checksums)?;
    native_source_feedback_decision(batch, proof).map_err(RelayError::NativeFeedbackRejected)
}

fn validate_native_feedback_ready(step: &RelayStep) -> Result<()> {
    require_durable(
        step.source_ack_boundary.all_publish_acks_durable,
        "all_publish_acks_durable",
    )?;
    require_durable(
        step.source_ack_boundary.durable_lsn_covers_commit,
        "durable_lsn_covers_commit",
    )?;
    require_durable(
        step.source_ack_boundary
            .checkpoint_recorded_before_source_ack,
        "checkpoint_recorded_before_source_ack",
    )
}

fn require_durable(value: bool, field: &'static str) -> Result<()> {
    if value {
        Ok(())
    } else {
        Err(RelayError::NativeFeedbackProofNotDurable { field })
    }
}

fn native_destinations(destinations: &[PublishDestination]) -> Vec<NativePublishDestination> {
    destinations
        .iter()
        .map(|destination| NativePublishDestination {
            topic: destination.topic.clone(),
            partition: destination.partition,
        })
        .collect()
}
