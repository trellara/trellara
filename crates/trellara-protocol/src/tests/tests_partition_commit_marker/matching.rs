use super::*;

#[test]
fn commit_marker_rejects_manifest_with_same_counts_but_different_content() {
    let (_envelope, plan) = partitioned_multi_store_plan();
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");
    let mut changed_manifest = plan.manifest.clone();
    changed_manifest.affected_tables[0]
        .relation
        .as_mut()
        .expect("affected relation")
        .table = "sales_shadow".to_string();

    assert_eq!(
        changed_manifest.global_event_count,
        plan.manifest.global_event_count
    );
    assert_eq!(
        changed_manifest.partitions.len(),
        plan.manifest.partitions.len()
    );
    assert_ne!(
        changed_manifest.compute_checksum(),
        plan.manifest.compute_checksum()
    );
    assert!(!marker.matches_manifest(&changed_manifest));
}

#[test]
fn commit_marker_does_not_match_duplicate_manifest_partitions() {
    let (_envelope, mut plan) = partitioned_multi_store_plan();
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");
    plan.manifest
        .partitions
        .push(plan.manifest.partitions[0].clone());

    assert!(!marker.matches_manifest(&plan.manifest));
}

#[test]
fn commit_marker_does_not_match_manifest_event_count_mismatch() {
    let (_envelope, mut plan) = partitioned_multi_store_plan();
    let marker = TransactionCommitMarker::from_manifest(&plan.manifest).expect("commit marker");
    plan.manifest.partitions[0].event_count += 1;

    assert!(!marker.matches_manifest(&plan.manifest));
}
