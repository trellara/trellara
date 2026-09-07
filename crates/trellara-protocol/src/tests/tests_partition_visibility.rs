use super::*;

fn partitioned_plan() -> PartitionPlan {
    plan_partitioned_transaction(
        &multi_store_envelope(),
        &PartitionPlanConfig {
            partition_count: 8,
            key_column: "store_id".to_string(),
            null_key_policy: PartitionNullKeyPolicy::Quarantine,
            key_change_policy: PartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan")
}

#[test]
fn barrier_visibility_releases_after_commit_marker_and_all_chunks() {
    let plan = partitioned_plan();
    let commit_marker =
        TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");

    let decision = barrier_visibility_decision(&plan.manifest, Some(&commit_marker), &plan.chunks)
        .expect("visibility decision");

    assert!(matches!(
        decision,
        BarrierVisibilityDecision::GloballyVisible {
            transaction_id,
            source_commit_lsn,
            event_count: 4,
            changes,
        } if transaction_id == "tx-1"
            && source_commit_lsn == "0/16B6C50"
            && changes.iter().map(|change| change.total_order).collect::<Vec<_>>() == vec![1, 2, 3, 4]
    ));
}

#[test]
fn barrier_visibility_holds_without_commit_marker() {
    let plan = partitioned_plan();

    let decision = barrier_visibility_decision(&plan.manifest, None, &plan.chunks)
        .expect("visibility decision");

    assert!(matches!(
        decision,
        BarrierVisibilityDecision::Held {
            transaction_id,
            reason: BarrierHoldReason::CommitMarkerMissing,
            missing_partitions,
        } if transaction_id == "tx-1" && missing_partitions.is_empty()
    ));
}

#[test]
fn barrier_visibility_rejects_duplicate_chunks_before_commit_marker() {
    let plan = partitioned_plan();
    let mut chunks = plan.chunks.clone();
    chunks.push(plan.chunks[0].clone());

    assert!(matches!(
        barrier_visibility_decision(&plan.manifest, None, &chunks),
        Err(ProtocolError::DuplicatePartitionChunk {
            transaction_id,
            partition_id,
        }) if transaction_id == "tx-1" && partition_id == plan.chunks[0].partition_id
    ));
}

#[test]
fn barrier_visibility_rejects_unlisted_chunks_before_commit_marker() {
    let plan = partitioned_plan();
    let mut chunks = plan.chunks.clone();
    let mut unlisted_chunk = plan.chunks[0].clone();
    unlisted_chunk.partition_id = 999;
    chunks.push(unlisted_chunk);

    assert!(matches!(
        barrier_visibility_decision(&plan.manifest, None, &chunks),
        Err(ProtocolError::PartitionNotInManifest {
            transaction_id,
            partition_id: 999,
        }) if transaction_id == "tx-1"
    ));
}

#[test]
fn barrier_visibility_holds_until_every_manifest_partition_arrives() {
    let plan = partitioned_plan();
    let commit_marker =
        TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");
    let missing_partition = plan.chunks[0].partition_id;
    let available_chunks = plan.chunks[1..].to_vec();

    let decision =
        barrier_visibility_decision(&plan.manifest, Some(&commit_marker), &available_chunks)
            .expect("visibility decision");

    assert!(matches!(
        decision,
        BarrierVisibilityDecision::Held {
            transaction_id,
            reason: BarrierHoldReason::PartitionChunksMissing,
            missing_partitions,
        } if transaction_id == "tx-1" && missing_partitions == vec![missing_partition]
    ));
}

#[test]
fn barrier_visibility_rejects_commit_marker_manifest_mismatch() {
    let mut plan = partitioned_plan();
    let commit_marker =
        TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");
    plan.manifest.global_event_count += 1;

    assert!(matches!(
        barrier_visibility_decision(&plan.manifest, Some(&commit_marker), &plan.chunks),
        Err(ProtocolError::CommitMarkerManifestMismatch { transaction_id })
            if transaction_id == "tx-1"
    ));
}

#[test]
fn partition_visibility_marks_cross_partition_chunk_as_local_only() {
    let plan = partitioned_plan();
    let partition_id = partition_for_key(b"store-104", 8);
    let chunk = plan
        .chunks
        .iter()
        .find(|chunk| chunk.partition_id == partition_id)
        .expect("store-104 chunk");

    let decision =
        partition_visibility_decision(&plan.manifest, chunk).expect("partition decision");

    assert!(matches!(
        decision,
        PartitionVisibilityDecision::PartitionLocalVisible {
            transaction_id,
            partition_id: actual_partition,
            global_complete: false,
            view,
        } if transaction_id == "tx-1"
            && actual_partition == partition_id
            && view.changes.iter().map(|change| change.total_order).collect::<Vec<_>>() == vec![1, 3]
    ));
}
