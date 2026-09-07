use super::{PartitionLoadObservation, PartitionRebalancePlanInput};
use crate::{parse_lsn, ProtocolError};

pub(super) fn validate_rebalance_input(
    input: &PartitionRebalancePlanInput,
) -> Result<(), ProtocolError> {
    if input.expected_partition_count == 0 {
        return Err(ProtocolError::InvalidPartitionCount);
    }
    reject_duplicate_observations(input)?;
    for observation in &input.observations {
        reject_out_of_range_observation(input, observation)?;
        validate_lsn_pair(observation)?;
    }
    Ok(())
}

fn reject_duplicate_observations(input: &PartitionRebalancePlanInput) -> Result<(), ProtocolError> {
    let mut observed = input
        .observations
        .iter()
        .map(|observation| observation.partition_id)
        .collect::<Vec<_>>();
    observed.sort_unstable();
    for window in observed.windows(2) {
        if window[0] == window[1] {
            return invalid_rebalance_plan(format!(
                "duplicate observation for partition {}",
                window[0]
            ));
        }
    }
    Ok(())
}

fn reject_out_of_range_observation(
    input: &PartitionRebalancePlanInput,
    observation: &PartitionLoadObservation,
) -> Result<(), ProtocolError> {
    if observation.partition_id < input.expected_partition_count {
        return Ok(());
    }
    invalid_rebalance_plan(format!(
        "partition {} is outside expected range 0..{}",
        observation.partition_id,
        input.expected_partition_count.saturating_sub(1)
    ))
}

fn validate_lsn_pair(observation: &PartitionLoadObservation) -> Result<(), ProtocolError> {
    let applied_lsn = parse_lsn(&observation.applied_lsn)?;
    let durable_lsn = parse_lsn(&observation.durable_lsn)?;
    if applied_lsn <= durable_lsn {
        return Ok(());
    }
    invalid_rebalance_plan(format!(
        "partition {} applied LSN {} is ahead of durable LSN {}",
        observation.partition_id, observation.applied_lsn, observation.durable_lsn
    ))
}

fn invalid_rebalance_plan<T>(reason: String) -> Result<T, ProtocolError> {
    Err(ProtocolError::InvalidManifestField {
        field: "partition_rebalance_plan",
        reason,
    })
}
