use super::fixtures::envelope_with;
use super::*;

#[test]
fn partitioned_scale_readiness_marks_dml_only_safe() {
    let envelope = envelope_with("tx-dml", "0/16B6B00", "0/16B6C50", vec![sample_change(1)]);

    let readiness = envelope.partitioned_scale_readiness();

    assert_eq!(readiness.transaction_id, "tx-dml");
    assert_eq!(readiness.boundary_kind, TransactionBoundaryKind::DmlOnly);
    assert_eq!(
        readiness.decision,
        PartitionedScaleDecision::PartitionParallelDml
    );
    assert!(readiness.partition_parallel_safe);
    assert!(!readiness.requires_ddl_barrier);
    assert!(!readiness.dml_replay_after_ddl_barrier_required);
    assert!(readiness.manifest_evidence.is_none());
    assert_eq!(
        readiness.reason,
        "DML-only transaction can be partitioned without a DDL barrier"
    );
}

#[test]
fn partitioned_scale_readiness_marks_ddl_only_as_barrier_path() {
    let mut envelope = envelope_with("tx-ddl", "0/16B6B00", "0/16B6C50", Vec::new());
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-ddl",
        1,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let readiness = envelope.partitioned_scale_readiness();

    assert_eq!(readiness.boundary_kind, TransactionBoundaryKind::DdlOnly);
    assert_eq!(
        readiness.decision,
        PartitionedScaleDecision::DdlBarrierRequired
    );
    assert!(!readiness.partition_parallel_safe);
    assert!(readiness.requires_ddl_barrier);
    assert!(!readiness.dml_replay_after_ddl_barrier_required);
    assert!(readiness.manifest_evidence.is_none());
    assert_eq!(
        readiness.reason,
        "DDL-only transaction must use the DDL barrier path"
    );
}

#[test]
fn partitioned_scale_readiness_marks_mixed_ddl_dml_as_replay_required() {
    let mut envelope = envelope_with("tx-mixed", "0/16B6B00", "0/16B6C50", vec![sample_change(2)]);
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-mixed",
        1,
        RelationId::new(16_384, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let readiness = envelope.partitioned_scale_readiness();

    assert_eq!(
        readiness.boundary_kind,
        TransactionBoundaryKind::MixedDdlAndDml
    );
    assert_eq!(
        readiness.decision,
        PartitionedScaleDecision::DdlBarrierRequired
    );
    assert!(!readiness.partition_parallel_safe);
    assert!(readiness.requires_ddl_barrier);
    assert!(readiness.dml_replay_after_ddl_barrier_required);
    assert!(readiness.manifest_evidence.is_none());
    assert_eq!(
        readiness.reason,
        "mixed DDL and DML transaction must apply DDL barrier before partitioned DML replay"
    );
}

#[test]
fn partitioned_scale_readiness_includes_manifest_evidence() {
    let mut envelope = multi_store_envelope();
    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    envelope.manifest = Some(plan.manifest.clone());
    envelope.finalize_checksum();

    let readiness = envelope.partitioned_scale_readiness();
    let evidence = readiness.manifest_evidence.expect("manifest evidence");

    assert_eq!(
        evidence.boundary_mode,
        ManifestBoundaryMode::PartitionedScale
    );
    assert_eq!(evidence.manifest_checksum, plan.manifest.compute_checksum());
    assert_eq!(evidence.global_event_count, 4);
    assert_eq!(evidence.manifest_event_count, 4);
    assert_eq!(evidence.envelope_event_count, 4);
    assert!(evidence.event_count_coverage);
    assert_eq!(
        evidence.participating_partition_count,
        plan.manifest.partitions.len()
    );
    assert_eq!(
        evidence.participating_partition_ids,
        plan.manifest
            .partitions
            .iter()
            .map(|partition| partition.id)
            .collect::<Vec<_>>()
    );
    assert_eq!(evidence.source_commit_lsn, "0/16B6C50");
    assert_eq!(
        evidence.visibility_contract,
        "global visibility waits for manifest, commit marker, and every participating partition"
    );
}

#[test]
fn partitioned_scale_readiness_marks_manifest_event_count_mismatch() {
    let mut envelope = multi_store_envelope();
    let mut plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    plan.manifest.global_event_count += 1;
    envelope.manifest = Some(plan.manifest);
    envelope.finalize_checksum();

    let evidence = envelope
        .partitioned_scale_readiness()
        .manifest_evidence
        .expect("manifest evidence");

    assert_eq!(evidence.global_event_count, 5);
    assert_eq!(evidence.manifest_event_count, 4);
    assert_eq!(evidence.envelope_event_count, 4);
    assert!(!evidence.event_count_coverage);
}
