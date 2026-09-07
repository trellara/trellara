use super::*;

#[test]
fn barrier_reconstruction_rejects_duplicate_manifest_partitions() {
    let mut plan = partition_plan(8);
    let duplicate_partition_id = plan.manifest.partitions[0].id;
    plan.manifest
        .partitions
        .push(plan.manifest.partitions[0].clone());

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::DuplicateManifestPartition { partition_id, .. })
            if partition_id == duplicate_partition_id
    ));
}

#[test]
fn barrier_reconstruction_rejects_manifest_event_count_mismatch() {
    let mut plan = partition_plan(8);
    plan.manifest.global_event_count += 1;

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::ManifestEventCountMismatch {
            transaction_id,
            expected,
            actual,
        }) if transaction_id == "tx-1"
            && expected == multi_store_envelope().changes.len() as u32 + 1
            && actual == multi_store_envelope().changes.len() as u32
    ));
}

#[test]
fn barrier_reconstruction_rejects_affected_table_missing_relation() {
    let mut plan = partition_plan(8);
    plan.manifest.affected_tables[0].relation = None;

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::ManifestAffectedTableMissingRelation {
            transaction_id,
            index: 0,
        }) if transaction_id == "tx-1"
    ));
}

#[test]
fn barrier_reconstruction_rejects_empty_affected_table() {
    let mut plan = partition_plan(8);
    plan.manifest.affected_tables[0].event_count = 0;

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::ManifestAffectedTableEmpty {
            transaction_id,
            relation,
        }) if transaction_id == "tx-1" && relation == "public.sales"
    ));
}

#[test]
fn barrier_reconstruction_rejects_invalid_manifest_partition_order_ranges() {
    for field in [
        "partitions[].first_total_order",
        "partitions[].last_total_order",
        "partitions[].total_order_range",
    ] {
        let mut plan = partition_plan(8);
        match field {
            "partitions[].first_total_order" => {
                plan.manifest.partitions[0].first_total_order = 0;
            }
            "partitions[].last_total_order" => {
                plan.manifest.partitions[0].last_total_order = 0;
            }
            "partitions[].total_order_range" => {
                let last = plan.manifest.partitions[0].last_total_order;
                plan.manifest.partitions[0].first_total_order = last + 1;
            }
            _ => unreachable!("test fields are exhaustive"),
        }

        assert!(matches!(
            reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
            Err(ProtocolError::InvalidManifestField { field: actual, .. })
                if actual == field
        ));
    }
}

#[test]
fn barrier_reconstruction_rejects_manifest_total_order_range_tampering() {
    let mut plan = partition_plan(8);
    let partition_id = plan.manifest.partitions[0].id;
    let actual_first = plan.manifest.partitions[0].first_total_order;
    let actual_last = plan.manifest.partitions[0].last_total_order;
    plan.manifest.partitions[0].last_total_order = actual_last + 1;

    assert!(matches!(
        reconstruct_barrier_transaction(&plan.manifest, &plan.chunks),
        Err(ProtocolError::PartitionTotalOrderRangeMismatch {
            partition_id: actual_partition,
            expected_first,
            expected_last,
            actual_first: found_first,
            actual_last: found_last,
        }) if actual_partition == partition_id
            && expected_first == actual_first
            && expected_last == actual_last + 1
            && found_first == actual_first
            && found_last == actual_last
    ));
}
