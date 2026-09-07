use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionBoundaryKind {
    Empty,
    DmlOnly,
    DdlOnly,
    MixedDdlAndDml,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PartitionedScaleDecision {
    EmptyTransaction,
    PartitionParallelDml,
    DdlBarrierRequired,
}

impl fmt::Display for PartitionedScaleDecision {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            PartitionedScaleDecision::EmptyTransaction => "empty_transaction",
            PartitionedScaleDecision::PartitionParallelDml => "partition_parallel_dml",
            PartitionedScaleDecision::DdlBarrierRequired => "ddl_barrier_required",
        })
    }
}
