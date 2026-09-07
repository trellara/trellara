use super::*;

#[test]
fn partition_local_view_rejects_duplicate_manifest_partitions() {
    let mut plan = partition_plan(8);
    let duplicate_partition_id = plan.manifest.partitions[0].id;
    plan.manifest
        .partitions
        .push(plan.manifest.partitions[0].clone());

    assert!(matches!(
        partition_local_view(&plan.manifest, &plan.chunks[0]),
        Err(ProtocolError::DuplicateManifestPartition { partition_id, .. })
            if partition_id == duplicate_partition_id
    ));
}

#[test]
fn partition_local_view_rejects_manifest_event_count_mismatch() {
    let mut plan = partition_plan(8);
    plan.manifest.partitions[0].event_count += 1;

    assert!(matches!(
        partition_local_view(&plan.manifest, &plan.chunks[0]),
        Err(ProtocolError::ManifestEventCountMismatch {
            transaction_id,
            expected,
            actual,
        }) if transaction_id == "tx-1"
            && expected == multi_store_envelope().changes.len() as u32
            && actual == multi_store_envelope().changes.len() as u32 + 1
    ));
}

#[test]
fn partition_local_view_rejects_duplicate_total_order_inside_verified_chunk() {
    let plan = duplicate_total_order_plan();

    assert!(matches!(
        partition_local_view(&plan.manifest, &plan.chunks[0]),
        Err(ProtocolError::DuplicateTransactionEventOrder { total_order: 3 })
    ));
}

#[test]
fn partition_local_view_rejects_manifest_total_order_range_tampering() {
    let mut plan = partition_plan(8);
    let chunk = plan
        .chunks
        .iter()
        .find(|chunk| {
            chunk
                .changes
                .first()
                .is_some_and(|change| change.total_order > 1)
        })
        .expect("chunk with non-initial first event")
        .clone();
    let partition_id = chunk.partition_id;
    let manifest_partition = plan
        .manifest
        .partitions
        .iter_mut()
        .find(|partition| partition.id == partition_id)
        .expect("manifest partition");
    let actual_first = manifest_partition.first_total_order;
    let actual_last = manifest_partition.last_total_order;
    manifest_partition.first_total_order = actual_first - 1;

    assert!(matches!(
        partition_local_view(&plan.manifest, &chunk),
        Err(ProtocolError::PartitionTotalOrderRangeMismatch {
            partition_id: actual_partition,
            expected_first,
            expected_last,
            actual_first: found_first,
            actual_last: found_last,
        }) if actual_partition == partition_id
            && expected_first == actual_first - 1
            && expected_last == actual_last
            && found_first == actual_first
            && found_last == actual_last
    ));
}

fn duplicate_total_order_plan() -> PartitionPlan {
    let mut plan = partition_plan(1);
    plan.chunks[0].changes[1].total_order = plan.chunks[0].changes[2].total_order;
    plan.chunks[0].finalize_checksum();
    plan.manifest.partitions[0].checksum = plan.chunks[0].checksum;
    plan
}
