use super::*;

pub(in crate::tests) fn healthy_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        slot_name: "slot-a".to_string(),
        exists: true,
        plugin: Some("test_decoding".to_string()),
        expected_plugin: "test_decoding".to_string(),
        active: Some(false),
        failover: None,
        synced: None,
        inactive_since: None,
        idle_replication_slot_timeout: None,
        restart_lsn: Some("0/16B6B00".to_string()),
        confirmed_flush_lsn: Some("0/16B6B00".to_string()),
        retained_wal_bytes: Some(0),
        wal_status: Some("reserved".to_string()),
        safe_wal_size_bytes: Some(1_000_000),
        invalidation_reason: None,
        wal_headroom: None,
        transaction_id_wraparound: None,
        xmin_horizon: None,
        issues: Vec::new(),
    }
}

pub(in crate::tests) fn active_healthy_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        active: Some(true),
        ..healthy_slot()
    }
}

pub(in crate::tests) fn missing_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        slot_name: "slot-a".to_string(),
        exists: false,
        plugin: None,
        expected_plugin: "pgoutput".to_string(),
        active: None,
        failover: None,
        synced: None,
        inactive_since: None,
        idle_replication_slot_timeout: None,
        restart_lsn: None,
        confirmed_flush_lsn: None,
        retained_wal_bytes: None,
        wal_status: None,
        safe_wal_size_bytes: None,
        invalidation_reason: None,
        wal_headroom: None,
        transaction_id_wraparound: None,
        xmin_horizon: None,
        issues: vec!["replication slot does not exist".to_string()],
    }
}

pub(in crate::tests) fn high_retention_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        retained_wal_bytes: Some(10_000),
        ..healthy_slot()
    }
}

pub(in crate::tests) fn low_wal_headroom_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        active: Some(true),
        retained_wal_bytes: Some(10_000),
        safe_wal_size_bytes: Some(50_400_000),
        wal_headroom: Some(trellara_pg_capture::WalHeadroomProjection {
            source: "pg_current_wal_lsn_sample".to_string(),
            safe_wal_size_bytes: 50_400_000,
            wal_bytes_per_second: 1_000,
            sample_ms: 2_000,
            headroom_seconds: Some(50_400),
        }),
        ..healthy_slot()
    }
}

pub(in crate::tests) fn transaction_id_wraparound_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        active: Some(true),
        transaction_id_wraparound: Some(trellara_pg_capture::TransactionIdWraparoundStatus {
            oldest_database: Some("app".to_string()),
            oldest_datfrozenxid_age: Some(180_000_000),
            autovacuum_freeze_max_age: Some(200_000_000),
            remaining_transactions: Some(20_000_000),
            usage_percent: Some(90),
        }),
        ..healthy_slot()
    }
}

pub(in crate::tests) fn xmin_horizon_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        active: Some(true),
        xmin_horizon: Some(trellara_pg_capture::XminHorizonStatus {
            slot_catalog_xmin: Some("720".to_string()),
            oldest_catalog_xmin_slot: Some("slot-a".to_string()),
            oldest_catalog_xmin: Some("720".to_string()),
            oldest_catalog_xmin_age: Some(1_000_000),
            stale_catalog_xmin_slot_count: 1,
            long_running_transaction_count: 2,
            oldest_transaction_age_seconds: Some(3_600),
            prepared_transaction_count: 1,
            oldest_prepared_transaction_age_seconds: Some(7_200),
            hot_standby_feedback_replica_count: 1,
            oldest_hot_standby_feedback_xmin_age: Some(500_000),
        }),
        ..healthy_slot()
    }
}

pub(in crate::tests) fn lost_wal_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
            wal_status: Some("lost".to_string()),
            safe_wal_size_bytes: Some(0),
            invalidation_reason: Some("wal_removed".to_string()),
            issues: vec![
                "replication slot invalidated: wal_removed; recreate the slot and reseed affected targets"
                    .to_string(),
                "replication slot WAL is lost; recreate the slot and reseed affected targets"
                    .to_string(),
            ],
            ..healthy_slot()
        }
}

pub(in crate::tests) fn wal_lost_without_invalidation_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        wal_status: Some("lost".to_string()),
        safe_wal_size_bytes: Some(0),
        issues: vec![
            "replication slot WAL is lost; recreate the slot and reseed affected targets"
                .to_string(),
        ],
        ..healthy_slot()
    }
}

pub(in crate::tests) fn unreserved_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
            wal_status: Some("unreserved".to_string()),
            safe_wal_size_bytes: Some(512),
            issues: vec![
                "replication slot WAL is unreserved and at risk of loss; drain or reseed before continuing"
                    .to_string(),
            ],
            ..healthy_slot()
        }
}

pub(in crate::tests) fn exhausted_safe_wal_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        safe_wal_size_bytes: Some(0),
        issues: vec![
            "replication slot safe WAL size is exhausted; drain or reseed before continuing"
                .to_string(),
        ],
        ..healthy_slot()
    }
}

pub(in crate::tests) fn plugin_mismatch_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        plugin: Some("test_decoding".to_string()),
        expected_plugin: "pgoutput".to_string(),
        issues: vec!["replication slot plugin is test_decoding, expected pgoutput".to_string()],
        ..healthy_slot()
    }
}

pub(in crate::tests) fn failover_disabled_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        active: Some(true),
        failover: Some(false),
        synced: None,
        ..healthy_slot()
    }
}

pub(in crate::tests) fn failover_unsynced_slot() -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        active: Some(true),
        failover: Some(true),
        synced: Some(false),
        ..healthy_slot()
    }
}

pub(in crate::tests) fn subscription_stats_with_conflicts() -> SubscriptionConflictStats {
    SubscriptionConflictStats {
        subscription_id: "42".to_string(),
        subscription_name: "downstream_sales".to_string(),
        apply_error_count: 2,
        sync_error_count: 0,
        conflicts: trellara_pg_capture::LogicalReplicationConflictCounts {
            insert_exists: 1,
            update_missing: 3,
            delete_missing: 2,
            ..Default::default()
        },
        stats_reset: Some("2026-08-15 09:00:00+00".to_string()),
    }
}

pub(in crate::tests) fn subscription_stats_with_apply_errors() -> SubscriptionConflictStats {
    SubscriptionConflictStats {
        subscription_id: "43".to_string(),
        subscription_name: "downstream_inventory".to_string(),
        apply_error_count: 4,
        sync_error_count: 1,
        conflicts: trellara_pg_capture::LogicalReplicationConflictCounts::default(),
        stats_reset: None,
    }
}
