use super::*;

#[test]
fn logical_replication_conflict_counts_total_pg18_counters() {
    let counts = LogicalReplicationConflictCounts {
        insert_exists: 1,
        update_origin_differs: 2,
        update_exists: 3,
        update_missing: 4,
        delete_origin_differs: 5,
        delete_missing: 6,
        multiple_unique_conflicts: 7,
    };
    let stats = SubscriptionConflictStats {
        subscription_id: "42".to_string(),
        subscription_name: "downstream".to_string(),
        apply_error_count: 1,
        sync_error_count: 0,
        conflicts: counts,
        stats_reset: None,
    };

    assert_eq!(stats.total_conflicts(), 28);
}
