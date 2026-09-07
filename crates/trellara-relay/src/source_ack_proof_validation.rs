use crate::publish_proof::durable_destinations;
use crate::{RelayError, RelayStep, Result, SOURCE_ACK_BOUNDARY_CONTRACT};

pub(crate) fn validate_step_source_ack_proof(step: &RelayStep) -> Result<()> {
    let proof = &step.source_ack_boundary;
    validate_proof_field("contract", &SOURCE_ACK_BOUNDARY_CONTRACT, &proof.contract)?;
    validate_proof_field("source_id", &step.envelope.source_id, &proof.source_id)?;
    validate_proof_field("dataset_id", &step.envelope.dataset_id, &proof.dataset_id)?;
    validate_proof_field("commit_lsn", &step.envelope.commit_lsn, &proof.commit_lsn)?;
    validate_proof_field(
        "source_ack_lsn",
        &step.source_ack_lsn,
        &proof.source_ack_lsn,
    )?;
    validate_proof_field(
        "durable_publish_acks",
        &step.publish_acks.len(),
        &proof.durable_publish_acks,
    )?;
    validate_proof_field(
        "expected_publish_messages",
        &step.publish_acks.len(),
        &proof.expected_publish_messages,
    )?;
    validate_proof_field(
        "expected_publish_destination_count",
        &step.published_messages.len(),
        &proof.expected_publish_destination_count,
    )?;
    let durable_publish_destinations = durable_destinations(&step.publish_acks);
    validate_proof_field(
        "durable_publish_destination_count",
        &durable_publish_destinations.len(),
        &proof.durable_publish_destination_count,
    )?;
    validate_proof_field(
        "durable_publish_destinations",
        &durable_publish_destinations,
        &proof.durable_publish_destinations,
    )?;
    validate_proof_field(
        "expected_publish_destinations",
        &proof.durable_publish_destinations,
        &proof.expected_publish_destinations,
    )?;
    validate_proof_field(
        "publish_destinations_match",
        &(proof.expected_publish_destinations == proof.durable_publish_destinations),
        &proof.publish_destinations_match,
    )?;
    validate_proof_field(
        "last_publish_ack",
        &step.publish_acks.last(),
        &proof.last_publish_ack.as_ref(),
    )?;
    validate_proof_field(
        "all_publish_acks_durable",
        &true,
        &proof.all_publish_acks_durable,
    )?;
    validate_proof_field(
        "durable_lsn_covers_commit",
        &true,
        &proof.durable_lsn_covers_commit,
    )?;
    validate_proof_field(
        "checkpoint_recorded_before_source_ack",
        &true,
        &proof.checkpoint_recorded_before_source_ack,
    )
}

fn validate_proof_field<T>(field: &'static str, step: &T, proof: &T) -> Result<()>
where
    T: std::fmt::Debug + PartialEq,
{
    if step == proof {
        Ok(())
    } else {
        Err(RelayError::SourceAckProofMismatch {
            field,
            step: format!("{step:?}"),
            proof: format!("{proof:?}"),
        })
    }
}
