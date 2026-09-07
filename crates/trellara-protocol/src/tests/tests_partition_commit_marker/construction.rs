use super::*;

#[test]
fn commit_marker_summarizes_partition_manifest() {
    let (_envelope, plan) = partitioned_multi_store_plan();

    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");

    assert_eq!(marker.transaction_id, "tx-1");
    assert_eq!(marker.source_commit_lsn, "0/16B6C50");
    assert_eq!(marker.global_event_count, 4);
    assert_eq!(
        marker.participating_partition_count,
        plan.manifest.partitions.len() as u32
    );
    assert_eq!(marker.manifest_checksum, plan.manifest.compute_checksum());
    assert!(marker.matches_manifest(&plan.manifest));
}
