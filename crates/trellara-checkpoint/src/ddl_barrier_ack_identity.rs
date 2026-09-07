use crate::{CheckpointError, DdlBarrier, DdlBarrierAck, Result};

pub(crate) fn validate_ack_boundaries(barrier: &DdlBarrier, acks: &[DdlBarrierAck]) -> Result<()> {
    for ack in acks {
        if ack.source_id == barrier.source_id
            && ack.database_id == barrier.database_id
            && ack.dataset_id == barrier.dataset_id
            && ack.barrier_id == barrier.barrier_id
        {
            continue;
        }
        return Err(CheckpointError::Store(format!(
            "DDL barrier {} ack for {} belongs to a different barrier identity",
            barrier.barrier_id, ack.sink
        )));
    }
    Ok(())
}
