use trellara_checkpoint::{DdlBarrierLookup, DdlBarrierStore, DdlBarrierSummary};
use trellara_protocol::TransactionEnvelope;

use crate::ddl_protocol::target_ddl_barrier_from_envelope_with_requirements;
use crate::{ApplyError, Result, TargetDdlBarrierRequirements};

pub async fn record_target_ddl_barrier_from_envelope<S: DdlBarrierStore + ?Sized>(
    store: &S,
    envelope: &TransactionEnvelope,
    requirements: TargetDdlBarrierRequirements,
) -> Result<Option<DdlBarrierSummary>> {
    let Some(barrier) = target_ddl_barrier_from_envelope_with_requirements(envelope, requirements)?
    else {
        return Ok(None);
    };
    let lookup = DdlBarrierLookup::from_barrier(&barrier);
    let barrier_id = barrier.barrier_id.clone();
    store.record_ddl_barrier(barrier).await?;
    store
        .ddl_barrier_summary(&lookup, &barrier_id)
        .await?
        .ok_or(ApplyError::MissingDdlField {
            field: "ddl_barrier_summary",
        })
        .map(Some)
}
