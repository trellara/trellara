use super::fixtures::envelope_with;
use super::*;

#[test]
fn boundary_kind_classifies_dml_only_transactions() {
    let envelope = envelope_with("tx-dml", "0/16B6B00", "0/16B6C50", vec![sample_change(1)]);

    assert_eq!(envelope.boundary_kind(), TransactionBoundaryKind::DmlOnly);
    assert!(!envelope.requires_ddl_barrier());
    assert!(!envelope.contains_mixed_ddl_dml());
    assert_eq!(
        envelope.partitioned_scale_decision(),
        PartitionedScaleDecision::PartitionParallelDml
    );
    assert!(envelope.is_partition_parallel_safe());
}

#[test]
fn boundary_kind_classifies_empty_transactions() {
    let envelope = envelope_with("tx-empty", "0/16B6B00", "0/16B6C50", Vec::new());

    assert_eq!(envelope.boundary_kind(), TransactionBoundaryKind::Empty);
    assert!(!envelope.requires_ddl_barrier());
    assert!(!envelope.contains_mixed_ddl_dml());
    assert_eq!(
        envelope.partitioned_scale_decision(),
        PartitionedScaleDecision::EmptyTransaction
    );
    assert!(!envelope.is_partition_parallel_safe());
}

#[test]
fn partitioned_scale_decision_renders_as_stable_snake_case() {
    assert_eq!(
        PartitionedScaleDecision::PartitionParallelDml.to_string(),
        "partition_parallel_dml"
    );
    assert_eq!(
        PartitionedScaleDecision::DdlBarrierRequired.to_string(),
        "ddl_barrier_required"
    );
}
