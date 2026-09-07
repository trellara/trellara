#[path = "transaction_boundary_decision.rs"]
mod transaction_boundary_decision;
#[path = "transaction_boundary_key.rs"]
mod transaction_boundary_key;

pub use transaction_boundary_decision::{PartitionedScaleDecision, TransactionBoundaryKind};
pub use transaction_boundary_key::TransactionBoundaryKey;

use crate::{ProtocolError, TransactionEnvelope};

impl TransactionEnvelope {
    pub fn boundary_key(&self) -> Result<TransactionBoundaryKey, ProtocolError> {
        TransactionBoundaryKey::from_envelope(self)
    }

    pub fn boundary_kind(&self) -> TransactionBoundaryKind {
        match (self.changes.is_empty(), self.ddl_events.is_empty()) {
            (true, true) => TransactionBoundaryKind::Empty,
            (false, true) => TransactionBoundaryKind::DmlOnly,
            (true, false) => TransactionBoundaryKind::DdlOnly,
            (false, false) => TransactionBoundaryKind::MixedDdlAndDml,
        }
    }

    pub fn requires_ddl_barrier(&self) -> bool {
        !self.ddl_events.is_empty()
    }

    pub fn contains_mixed_ddl_dml(&self) -> bool {
        self.boundary_kind() == TransactionBoundaryKind::MixedDdlAndDml
    }

    pub fn partitioned_scale_decision(&self) -> PartitionedScaleDecision {
        match self.boundary_kind() {
            TransactionBoundaryKind::Empty => PartitionedScaleDecision::EmptyTransaction,
            TransactionBoundaryKind::DmlOnly => PartitionedScaleDecision::PartitionParallelDml,
            TransactionBoundaryKind::DdlOnly | TransactionBoundaryKind::MixedDdlAndDml => {
                PartitionedScaleDecision::DdlBarrierRequired
            }
        }
    }

    pub fn is_partition_parallel_safe(&self) -> bool {
        self.partitioned_scale_decision() == PartitionedScaleDecision::PartitionParallelDml
    }
}
