use std::collections::BTreeSet;

use bytes::Bytes;

use super::*;

#[test]
fn maintenance_plan_compacts_small_files_before_snapshot_and_orphan_cleanup() {
    let target =
        IcebergTableIdentifier::new(["analytics", "retail"], "orders_cdc").expect("target");
    let policy = IcebergMaintenancePolicy {
        small_file_threshold_bytes: 100,
        target_file_size_bytes: 1_000,
        minimum_small_file_count: 3,
        snapshot_max_age_ms: 10_000,
        retain_last_snapshots: 2,
        orphan_min_age_ms: 20_000,
    };
    let files = vec![
        maintenance_file("a", 10),
        maintenance_file("b", 20),
        maintenance_file("c", 30),
        maintenance_file("large", 500),
    ];
    let plan = build_maintenance_plan(&target, "prod", 101, &files, &policy, 1_800_000_000_000)
        .expect("maintenance plan");

    assert_eq!(
        plan.status,
        IcebergMaintenancePlanStatus::CompactionRequired
    );
    assert_eq!(plan.small_file_count, 3);
    assert_eq!(plan.small_file_bytes, 60);
    assert_eq!(
        plan.execution_order,
        vec![
            "rewrite_data_files",
            "expire_snapshots",
            "remove_orphan_files"
        ]
    );
    assert!(plan
        .rewrite_data_files_sql
        .as_deref()
        .expect("rewrite SQL")
        .contains("target-file-size-bytes', '1000'"));
    assert!(plan.expire_snapshots_sql.contains("retain_last => 2"));
    assert!(plan.remove_orphan_files_sql.contains("older_than"));
}

#[tokio::test]
async fn orphan_cleanup_preserves_catalog_and_pending_objects() {
    let store = super::object_store::FaultInjectingStore::default();
    for key in [
        "warehouse/orphan.parquet",
        "warehouse/referenced.parquet",
        "warehouse/pending.parquet",
        "warehouse/metadata.json",
    ] {
        upload_immutable_object(
            &store,
            key,
            format!("s3://lake/{key}"),
            Bytes::from(key.to_string()),
        )
        .await
        .expect("seed object");
    }
    let report = cleanup_orphan_objects(
        &store,
        &IcebergOrphanCleanupRequest {
            prefix: "warehouse/".to_string(),
            now_ms: 100,
            minimum_age_ms: 10,
            referenced_object_keys: BTreeSet::from(["warehouse/referenced.parquet".to_string()]),
            pending_object_keys: BTreeSet::from(["warehouse/pending.parquet".to_string()]),
            dry_run: false,
        },
    )
    .await
    .expect("cleanup");

    assert_eq!(report.candidate_count, 1);
    assert_eq!(report.deleted_object_keys, vec!["warehouse/orphan.parquet"]);
    assert!(report
        .protected_object_keys
        .contains(&"warehouse/referenced.parquet".to_string()));
    assert!(report
        .protected_object_keys
        .contains(&"warehouse/pending.parquet".to_string()));
    assert!(report
        .protected_object_keys
        .contains(&"warehouse/metadata.json".to_string()));
}

#[tokio::test]
async fn orphan_cleanup_rejects_bucket_root_and_non_directory_prefixes() {
    let store = super::object_store::FaultInjectingStore::default();
    for prefix in ["", "warehouse", "/warehouse/", "warehouse/../other/"] {
        let error = cleanup_orphan_objects(
            &store,
            &IcebergOrphanCleanupRequest {
                prefix: prefix.to_string(),
                now_ms: 100,
                minimum_age_ms: 10,
                referenced_object_keys: BTreeSet::new(),
                pending_object_keys: BTreeSet::new(),
                dry_run: true,
            },
        )
        .await
        .expect_err("unsafe cleanup prefix");
        assert!(matches!(
            error,
            IcebergIntegrationError::UnsafeMaintenancePlan { .. }
        ));
    }
}

fn maintenance_file(name: &str, size: u64) -> IcebergMaintenanceFile {
    IcebergMaintenanceFile {
        file_uri: format!("s3://lake/{name}.parquet"),
        file_size_in_bytes: size,
        record_count: Some(1),
    }
}
