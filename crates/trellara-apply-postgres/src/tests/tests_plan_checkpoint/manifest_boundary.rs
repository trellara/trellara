use super::*;

#[test]
fn checkpoint_evidence_rejects_manifest_commit_boundary_mismatch() {
    let mut envelope = envelope("tx-manifest-mismatch", "0/16B6C50");
    let plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "amount_cents".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    envelope.commit_lsn = "0/16B6C51".to_string();
    envelope.manifest = Some(plan.manifest);

    let error = validate_apply_checkpoint_evidence(&envelope)
        .expect_err("manifest commit mismatch rejected");

    assert!(matches!(
        error,
        ApplyError::CheckpointBoundaryMismatch {
            field: "commit_lsn",
            ..
        }
    ));
}

#[test]
fn checkpoint_evidence_rejects_manifest_transaction_boundary_mismatch() {
    let mut envelope = envelope("tx-manifest-transaction", "0/16B6C50");
    let mut plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "amount_cents".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    plan.manifest.transaction_id = "tx-other".to_string();
    envelope.manifest = Some(plan.manifest);

    let error = validate_apply_checkpoint_evidence(&envelope)
        .expect_err("manifest transaction mismatch rejected");

    assert!(matches!(
        error,
        ApplyError::CheckpointBoundaryMismatch {
            field: "transaction_id",
            envelope,
            manifest,
        } if envelope == "tx-manifest-transaction" && manifest == "tx-other"
    ));
}

#[test]
fn checkpoint_evidence_rejects_empty_partition_manifest() {
    let mut envelope = envelope("tx-empty-manifest", "0/16B6C50");
    let mut plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "amount_cents".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    plan.manifest.partitions.clear();
    envelope.manifest = Some(plan.manifest);

    let error = validate_apply_checkpoint_evidence(&envelope)
        .expect_err("empty partition manifest rejected");

    assert!(matches!(
        error,
        ApplyError::Protocol(ProtocolError::InvalidPartitionCount)
    ));
}

#[test]
fn checkpoint_evidence_rejects_duplicate_manifest_partition_ids() {
    let mut envelope = envelope("tx-duplicate-manifest-partition", "0/16B6C50");
    let mut plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "amount_cents".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    let duplicate_partition_id = plan.manifest.partitions[0].id;
    plan.manifest
        .partitions
        .push(plan.manifest.partitions[0].clone());
    envelope.manifest = Some(plan.manifest);

    let error = validate_apply_checkpoint_evidence(&envelope)
        .expect_err("duplicate manifest partition rejected");

    assert!(matches!(
        error,
        ApplyError::Protocol(ProtocolError::DuplicateManifestPartition {
            partition_id,
            ..
        }) if partition_id == duplicate_partition_id
    ));
}

#[test]
fn checkpoint_evidence_rejects_affected_table_count_mismatch() {
    let mut envelope = envelope("tx-affected-table-mismatch", "0/16B6C50");
    let mut plan = plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "amount_cents".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");
    plan.manifest.affected_tables[0].event_count += 1;
    envelope.manifest = Some(plan.manifest);

    let error = validate_apply_checkpoint_evidence(&envelope)
        .expect_err("affected table count mismatch rejected");

    assert!(matches!(
        error,
        ApplyError::Protocol(ProtocolError::ManifestAffectedTableCountMismatch {
            transaction_id,
            ..
        }) if transaction_id == "tx-affected-table-mismatch"
    ));
}

#[test]
fn checkpoint_evidence_rejects_invalid_manifest_boundary_modes() {
    for boundary_mode in [ManifestBoundaryMode::Unspecified as i32, 99] {
        let mut envelope = envelope("tx-invalid-boundary-mode", "0/16B6C50");
        let mut plan = plan_partitioned_transaction(
            &envelope,
            &PartitionPlanConfig {
                partition_count: 8,
                key_column: "amount_cents".to_string(),
                null_key_policy: PartitionNullKeyPolicy::Quarantine,
                key_change_policy: PartitionKeyChangePolicy::Quarantine,
            },
        )
        .expect("partition plan");
        plan.manifest.boundary_mode = boundary_mode;
        envelope.manifest = Some(plan.manifest);

        let error = validate_apply_checkpoint_evidence(&envelope)
            .expect_err("invalid manifest boundary mode rejected");

        assert!(matches!(
            error,
            ApplyError::InvalidCheckpointManifestBoundaryMode { boundary_mode: actual }
                if actual == boundary_mode
        ));
    }
}
