use super::*;

#[test]
fn partition_planning_rejects_ddl_events_until_barriers_support_them() {
    let mut envelope = envelope_with("tx-1", vec![sale_change(2, "store-104")]);
    envelope.ddl_events = vec![ddl_event("tx-1", 1)];
    envelope.finalize_checksum();

    assert!(matches!(
        plan_partitioned_transaction(&envelope, &default_partition_config()),
        Err(ProtocolError::UnsupportedDdlInManifestMode {
            boundary_mode,
            boundary_kind: TransactionBoundaryKind::MixedDdlAndDml,
            partitioned_scale_decision: PartitionedScaleDecision::DdlBarrierRequired,
            ..
        }) if boundary_mode == "partitioned scale mode"
    ));
}

#[test]
fn partition_planning_rejects_ddl_only_boundary_with_explicit_kind() {
    let mut envelope = envelope_with("tx-ddl-only", Vec::new());
    envelope.ddl_events = vec![ddl_event("tx-ddl-only", 1)];
    envelope.finalize_checksum();

    assert!(matches!(
        plan_partitioned_transaction(&envelope, &default_partition_config()),
        Err(ProtocolError::UnsupportedDdlInManifestMode {
            boundary_mode,
            boundary_kind: TransactionBoundaryKind::DdlOnly,
            partitioned_scale_decision: PartitionedScaleDecision::DdlBarrierRequired,
            ..
        }) if boundary_mode == "partitioned scale mode"
    ));
}

#[test]
fn partition_planning_accepts_explicit_dml_replay_after_ddl_barrier() {
    let mut envelope = envelope_with("tx-1", vec![sale_change(2, "store-104")]);
    envelope.ddl_events = vec![ddl_event("tx-1", 1)];
    envelope.finalize_checksum();

    let plan = plan_partitioned_transaction(
        &envelope.dml_replay_after_ddl_barrier(),
        &default_partition_config(),
    )
    .expect("partition plan after DDL barrier");

    assert_eq!(plan.manifest.transaction_id, "tx-1");
    assert_eq!(plan.manifest.global_event_count, 1);
    assert_eq!(
        plan.chunks
            .iter()
            .map(|chunk| chunk.changes.len())
            .sum::<usize>(),
        1
    );
}

#[test]
fn strict_chunk_planning_rejects_ddl_events_until_barriers_support_them() {
    let mut envelope = envelope_with("tx-1", vec![sample_change(2)]);
    envelope.ddl_events = vec![ddl_event("tx-1", 1)];
    envelope.finalize_checksum();

    assert!(matches!(
        plan_strict_chunked_transaction(
            &envelope,
            &StrictChunkPlanConfig {
                max_changes_per_chunk: 1,
            },
        ),
        Err(ProtocolError::UnsupportedDdlInManifestMode {
            boundary_mode,
            boundary_kind: TransactionBoundaryKind::MixedDdlAndDml,
            partitioned_scale_decision: PartitionedScaleDecision::DdlBarrierRequired,
            ..
        }) if boundary_mode == "strict chunked transaction order"
    ));
}

#[test]
fn strict_chunk_planning_accepts_explicit_dml_replay_after_ddl_barrier() {
    let mut envelope = envelope_with("tx-1", vec![sample_change(2), sample_change(3)]);
    envelope.ddl_events = vec![ddl_event("tx-1", 1)];
    envelope.finalize_checksum();

    let plan = plan_strict_chunked_transaction(
        &envelope.dml_replay_after_ddl_barrier(),
        &StrictChunkPlanConfig {
            max_changes_per_chunk: 1,
        },
    )
    .expect("strict chunk plan after DDL barrier");

    assert_eq!(plan.manifest.transaction_id, "tx-1");
    assert_eq!(plan.manifest.global_event_count, 2);
    assert_eq!(plan.chunks.len(), 2);
}
