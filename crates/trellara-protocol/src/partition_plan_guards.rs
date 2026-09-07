use crate::{PartitionedScaleDecision, ProtocolError, TransactionEnvelope};

pub(crate) fn ensure_non_empty_transaction(
    envelope: &TransactionEnvelope,
) -> Result<(), ProtocolError> {
    if envelope.changes.is_empty() {
        Err(ProtocolError::EmptyTransaction {
            transaction_id: envelope.transaction_id.clone(),
        })
    } else {
        Ok(())
    }
}

pub(crate) fn ensure_no_ddl_events(
    envelope: &TransactionEnvelope,
    boundary_mode: &'static str,
) -> Result<(), ProtocolError> {
    let partitioned_scale_decision = envelope.partitioned_scale_decision();
    match partitioned_scale_decision {
        PartitionedScaleDecision::PartitionParallelDml
        | PartitionedScaleDecision::EmptyTransaction => Ok(()),
        PartitionedScaleDecision::DdlBarrierRequired => {
            Err(ProtocolError::UnsupportedDdlInManifestMode {
                transaction_id: envelope.transaction_id.clone(),
                boundary_mode,
                boundary_kind: envelope.boundary_kind(),
                partitioned_scale_decision,
            })
        }
    }
}
