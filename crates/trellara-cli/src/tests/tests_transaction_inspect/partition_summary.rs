use super::*;

#[test]
fn transaction_inspector_summarizes_tables_and_partition_manifest() {
    let summary = TransactionInspectSummary::from_envelope(&inspect_envelope());

    assert_eq!(summary.source_id, "source-a");
    assert_eq!(summary.database_id, "retail");
    assert_eq!(summary.dataset_id, "sales");
    assert_eq!(summary.transaction_id, "tx-inspect");
    assert_eq!(summary.begin_lsn, "0/16B6B00");
    assert_eq!(summary.commit_lsn, "0/16B6C50");
    assert_eq!(summary.event_count, 3);
    assert_eq!(summary.ddl_event_count, 0);
    assert_eq!(summary.source_event_count, 3);
    assert!(summary.ddl_events.is_empty());
    assert!(summary.dml_replay_after_ddl_barrier.is_none());
    assert_eq!(
        summary.transaction_boundary.status,
        TransactionBoundaryStatus::Verified
    );
    assert_eq!(
        summary.transaction_boundary.mode,
        "partitioned_scale_mode".to_string()
    );
    assert_eq!(
        summary.transaction_boundary.source_boundary_kind,
        "dml_only"
    );
    assert_eq!(
        summary.transaction_boundary.partitioned_scale_decision,
        "partition_parallel_dml"
    );
    assert!(summary.transaction_boundary.partition_parallel_safe);
    assert!(!summary.transaction_boundary.requires_ddl_barrier);
    assert!(
        !summary
            .transaction_boundary
            .dml_replay_after_ddl_barrier_required
    );
    assert_eq!(
        summary.transaction_boundary.partitioned_scale_reason,
        "DML-only transaction can be partitioned without a DDL barrier"
    );
    assert!(summary
        .transaction_boundary
        .visibility_contract
        .contains("partition-local consumers may read a lane earlier"));
    assert_eq!(
        summary.transaction_boundary.checksum_status,
        ChecksumStatus::Match
    );
    assert!(summary.transaction_boundary.manifest_barrier_required);
    assert!(summary.transaction_boundary.manifest_valid);
    assert!(summary
        .transaction_boundary
        .manifest_validation_error
        .is_none());
    assert_eq!(
        summary.transaction_boundary.global_event_count_matches,
        Some(true)
    );
    assert_eq!(
        summary.transaction_boundary.partition_event_count_matches,
        Some(true)
    );
    assert!(summary
        .transaction_boundary
        .partitioned_scale_manifest_checksum
        .is_some());
    assert_eq!(
        summary
            .transaction_boundary
            .partitioned_scale_manifest_event_count,
        Some(3)
    );
    assert_eq!(
        summary
            .transaction_boundary
            .partitioned_scale_envelope_event_count,
        Some(3)
    );
    assert_eq!(
        summary
            .transaction_boundary
            .partitioned_scale_event_count_coverage,
        Some(true)
    );
    assert_eq!(
        summary
            .transaction_boundary
            .partitioned_scale_participating_partition_ids,
        vec![0, 2]
    );
    assert_eq!(
        summary
            .transaction_boundary
            .partitioned_scale_visibility_contract
            .as_deref(),
        Some("global visibility waits for manifest, commit marker, and every participating partition")
    );
    assert_eq!(
        summary.transaction_boundary.participating_partition_count,
        2
    );
    assert_eq!(summary.affected_tables.len(), 2);
    assert_eq!(
        summary.affected_tables[0],
        TransactionInspectTable {
            relation: "public.orders".to_string(),
            event_count: 2,
            inserts: 1,
            updates: 0,
            deletes: 1,
            truncates: 0,
        }
    );
    assert_eq!(
        summary.affected_tables[1],
        TransactionInspectTable {
            relation: "public.payments".to_string(),
            event_count: 1,
            inserts: 0,
            updates: 1,
            deletes: 0,
            truncates: 0,
        }
    );
    let manifest = summary.partition_manifest.expect("manifest");
    assert_eq!(manifest.boundary_mode, "partitioned_scale_mode");
    assert_eq!(manifest.global_event_count, 3);
    assert_eq!(manifest.participating_partition_count, 2);
    assert_eq!(
        manifest
            .partitions
            .iter()
            .map(|partition| partition.id)
            .collect::<Vec<_>>(),
        vec![0, 2]
    );
}
