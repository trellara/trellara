use super::fixtures::{additive_ddl, ddl_envelope};
use super::*;

#[test]
fn dml_replay_after_ddl_barrier_strips_ddl_and_refreshes_checksum() {
    let envelope = ddl_envelope(
        "tx-ddl-dml",
        vec![sample_change(2)],
        vec![additive_ddl("tx-ddl-dml", 1)],
    );

    let replay = envelope.dml_replay_after_ddl_barrier();

    assert!(replay.ddl_events.is_empty());
    assert_eq!(replay.changes.len(), 1);
    assert_eq!(replay.changes[0].total_order, 2);
    assert_eq!(replay.transaction_id, envelope.transaction_id);
    replay.verify_checksum().expect("replay checksum");
    assert_ne!(replay.checksum, envelope.checksum);
}

#[test]
fn boundary_kind_classifies_ddl_only_transactions() {
    let envelope = ddl_envelope("tx-ddl", Vec::new(), vec![additive_ddl("tx-ddl", 1)]);

    assert_eq!(envelope.boundary_kind(), TransactionBoundaryKind::DdlOnly);
    assert!(envelope.requires_ddl_barrier());
    assert!(!envelope.contains_mixed_ddl_dml());
    assert_eq!(
        envelope.partitioned_scale_decision(),
        PartitionedScaleDecision::DdlBarrierRequired
    );
    assert!(!envelope.is_partition_parallel_safe());
}

#[test]
fn boundary_kind_classifies_mixed_ddl_dml_transactions() {
    let envelope = ddl_envelope(
        "tx-ddl-dml",
        vec![sample_change(2)],
        vec![additive_ddl("tx-ddl-dml", 1)],
    );

    assert_eq!(
        envelope.boundary_kind(),
        TransactionBoundaryKind::MixedDdlAndDml
    );
    assert!(envelope.requires_ddl_barrier());
    assert!(envelope.contains_mixed_ddl_dml());
    assert_eq!(
        envelope.partitioned_scale_decision(),
        PartitionedScaleDecision::DdlBarrierRequired
    );
    assert!(!envelope.is_partition_parallel_safe());
}
