use super::*;

#[test]
fn commit_marker_rejects_duplicate_manifest_partitions() {
    let (_envelope, mut plan) = partitioned_multi_store_plan();
    let duplicate_partition_id = plan.manifest.partitions[0].id;
    plan.manifest
        .partitions
        .push(plan.manifest.partitions[0].clone());

    assert!(matches!(
        TransactionCommitMarker::from_manifest(&plan.manifest),
        Err(ProtocolError::DuplicateManifestPartition { partition_id, .. })
            if partition_id == duplicate_partition_id
    ));
}

#[test]
fn commit_marker_rejects_manifest_event_count_mismatch() {
    let (envelope, mut plan) = partitioned_multi_store_plan();
    plan.manifest.global_event_count += 1;

    assert!(matches!(
        TransactionCommitMarker::from_manifest(&plan.manifest),
        Err(ProtocolError::ManifestEventCountMismatch {
            transaction_id,
            expected,
            actual,
        }) if transaction_id == "tx-1"
            && expected == envelope.changes.len() as u32 + 1
            && actual == envelope.changes.len() as u32
    ));
}

#[test]
fn commit_marker_rejects_invalid_manifest_identity_fields() {
    for field in ["transaction_id", "source_commit_lsn", "global_event_count"] {
        let (_envelope, mut plan) = partitioned_multi_store_plan();
        match field {
            "transaction_id" => plan.manifest.transaction_id = " tx-1".to_string(),
            "source_commit_lsn" => plan.manifest.source_commit_lsn = "0/16B6C50 ".to_string(),
            "global_event_count" => plan.manifest.global_event_count = 0,
            _ => unreachable!("test fields are exhaustive"),
        }

        assert!(matches!(
            TransactionCommitMarker::from_manifest(&plan.manifest),
            Err(ProtocolError::InvalidManifestField { field: actual, .. })
                if actual == field
        ));
    }
}

#[test]
fn commit_marker_rejects_invalid_manifest_commit_timestamp() {
    for timestamp in [0, -1] {
        let (_envelope, mut plan) = partitioned_multi_store_plan();
        plan.manifest.source_commit_timestamp_ms = timestamp;

        assert!(matches!(
            TransactionCommitMarker::from_manifest(&plan.manifest),
            Err(ProtocolError::InvalidManifestField { field, .. })
                if field == "source_commit_timestamp_ms"
        ));
    }
}

#[test]
fn commit_marker_rejects_invalid_manifest_boundary_modes() {
    for boundary_mode in [ManifestBoundaryMode::Unspecified as i32, 99] {
        let (_envelope, mut plan) = partitioned_multi_store_plan();
        plan.manifest.boundary_mode = boundary_mode;

        assert!(matches!(
            TransactionCommitMarker::from_manifest(&plan.manifest),
            Err(ProtocolError::InvalidManifestField { field, .. }) if field == "boundary_mode"
        ));
    }
}

#[test]
fn commit_marker_rejects_empty_manifest_structural_fields() {
    for field in ["partitions", "affected_tables"] {
        let (_envelope, mut plan) = partitioned_multi_store_plan();
        match field {
            "partitions" => plan.manifest.partitions.clear(),
            "affected_tables" => plan.manifest.affected_tables.clear(),
            _ => unreachable!("test fields are exhaustive"),
        }

        assert!(matches!(
            TransactionCommitMarker::from_manifest(&plan.manifest),
            Err(ProtocolError::InvalidManifestField { field: actual, .. })
                if actual == field
        ));
    }
}

#[test]
fn commit_marker_rejects_invalid_manifest_partition_order_ranges() {
    for field in [
        "partitions[].first_total_order",
        "partitions[].last_total_order",
        "partitions[].total_order_range",
    ] {
        let (_envelope, mut plan) = partitioned_multi_store_plan();
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
            TransactionCommitMarker::from_manifest(&plan.manifest),
            Err(ProtocolError::InvalidManifestField { field: actual, .. })
                if actual == field
        ));
    }
}

#[test]
fn commit_marker_rejects_manifest_partition_span_past_global_event_count() {
    let (_envelope, mut plan) = partitioned_multi_store_plan();
    plan.manifest.partitions[0].last_total_order = plan.manifest.global_event_count + 1;

    assert!(matches!(
        TransactionCommitMarker::from_manifest(&plan.manifest),
        Err(ProtocolError::InvalidManifestField { field, reason })
            if field == "partitions[].last_total_order"
                && reason.contains("global_event_count")
    ));
}

#[test]
fn commit_marker_rejects_manifest_partition_count_larger_than_order_span() {
    let (_envelope, mut plan) = partitioned_multi_store_plan();
    plan.manifest.partitions[0].first_total_order = 1;
    plan.manifest.partitions[0].last_total_order = 1;
    plan.manifest.partitions[0].event_count = 2;

    assert!(matches!(
        TransactionCommitMarker::from_manifest(&plan.manifest),
        Err(ProtocolError::InvalidManifestField { field, reason })
            if field == "partitions[].event_count" && reason.contains("total_order range")
    ));
}

#[test]
fn commit_marker_rejects_manifest_total_order_span_without_first_event() {
    let manifest = manifest_with_impossible_total_order_span(vec![
        ManifestPartition {
            id: 0,
            event_count: 2,
            first_total_order: 2,
            last_total_order: 3,
            checksum: 11,
        },
        ManifestPartition {
            id: 1,
            event_count: 1,
            first_total_order: 2,
            last_total_order: 2,
            checksum: 22,
        },
    ]);

    assert!(matches!(
        TransactionCommitMarker::from_manifest(&manifest),
        Err(ProtocolError::InvalidManifestField { field, reason })
            if field == "partitions[].first_total_order"
                && reason.contains("first transaction event")
    ));
}

#[test]
fn commit_marker_rejects_affected_table_count_mismatch() {
    let (envelope, mut plan) = partitioned_multi_store_plan();
    plan.manifest.affected_tables[0].event_count -= 1;

    assert!(matches!(
        TransactionCommitMarker::from_manifest(&plan.manifest),
        Err(ProtocolError::ManifestAffectedTableCountMismatch {
            transaction_id,
            expected,
            actual,
        }) if transaction_id == "tx-1"
            && expected == envelope.changes.len() as u32
            && actual == envelope.changes.len() as u32 - 1
    ));
}

#[test]
fn commit_marker_rejects_duplicate_affected_tables() {
    let (_envelope, mut plan) = partitioned_multi_store_plan();
    plan.manifest
        .affected_tables
        .push(plan.manifest.affected_tables[0].clone());

    assert!(matches!(
        TransactionCommitMarker::from_manifest(&plan.manifest),
        Err(ProtocolError::DuplicateManifestAffectedTable {
            transaction_id,
            relation,
        }) if transaction_id == "tx-1" && relation == "public.sales"
    ));
}

fn manifest_with_impossible_total_order_span(
    partitions: Vec<ManifestPartition>,
) -> TransactionManifest {
    TransactionManifest {
        transaction_id: "tx-1".to_string(),
        source_commit_lsn: "0/16B6C50".to_string(),
        source_commit_timestamp_ms: 1_786_420_000_000,
        global_event_count: 3,
        partitions,
        affected_tables: vec![AffectedTable {
            relation: Some(RelationId::new(16_384, "public", "sales")),
            event_count: 3,
        }],
        boundary_mode: ManifestBoundaryMode::PartitionedScale as i32,
    }
}
