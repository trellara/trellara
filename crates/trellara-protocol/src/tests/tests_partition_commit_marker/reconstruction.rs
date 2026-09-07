use super::*;

#[test]
fn committed_barrier_reconstruction_requires_matching_commit_marker() {
    let (envelope, plan) = partitioned_multi_store_plan();
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");

    let changes = reconstruct_committed_barrier_transaction(&plan.manifest, &marker, &plan.chunks)
        .expect("committed reconstruct");

    assert_eq!(changes.len(), envelope.changes.len());
    assert_eq!(
        changes
            .iter()
            .map(|change| change.total_order)
            .collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}

#[test]
fn committed_barrier_reconstruction_rejects_mismatched_commit_marker() {
    let (_envelope, plan) = partitioned_multi_store_plan();
    let mut marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");
    marker.global_event_count += 1;

    let error = reconstruct_committed_barrier_transaction(&plan.manifest, &marker, &plan.chunks)
        .expect_err("mismatched marker");

    assert!(matches!(
        error,
        ProtocolError::CommitMarkerManifestMismatch { transaction_id }
            if transaction_id == "tx-1"
    ));
}

#[test]
fn committed_barrier_reconstruction_rejects_invalid_commit_marker_fields() {
    for field in [
        "transaction_id",
        "source_commit_lsn",
        "source_commit_timestamp_ms",
        "global_event_count",
        "participating_partition_count",
        "manifest_checksum",
    ] {
        let (_envelope, plan) = partitioned_multi_store_plan();
        let mut marker =
            TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");
        match field {
            "transaction_id" => marker.transaction_id = " tx-1".to_string(),
            "source_commit_lsn" => marker.source_commit_lsn = "0/16B6C50 ".to_string(),
            "source_commit_timestamp_ms" => marker.source_commit_timestamp_ms = 0,
            "global_event_count" => marker.global_event_count = 0,
            "participating_partition_count" => marker.participating_partition_count = 0,
            "manifest_checksum" => marker.manifest_checksum = 0,
            _ => unreachable!("test fields are exhaustive"),
        }

        assert!(matches!(
            reconstruct_committed_barrier_transaction(&plan.manifest, &marker, &plan.chunks),
            Err(ProtocolError::InvalidCommitMarkerField { field: actual, .. })
                if actual == field
        ));
    }
}

#[test]
fn barrier_reconstruction_rejects_duplicate_total_order_across_partitions() {
    let (_envelope, mut plan) = partitioned_multi_store_plan();
    let duplicate_total_order = plan.chunks[0].changes[0].total_order;
    plan.chunks[1].changes[0].total_order = duplicate_total_order;
    plan.chunks[1].finalize_checksum();
    plan.manifest.partitions[1].first_total_order = duplicate_total_order;
    plan.manifest.partitions[1].last_total_order = duplicate_total_order;
    plan.manifest.partitions[1].checksum = plan.chunks[1].checksum;
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");

    assert!(matches!(
        reconstruct_committed_barrier_transaction(&plan.manifest, &marker, &plan.chunks),
        Err(ProtocolError::DuplicateTransactionEventOrder { total_order })
            if total_order == duplicate_total_order
    ));
}
