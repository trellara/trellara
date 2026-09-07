use std::collections::BTreeSet;

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

use crate::{IcebergIntegrationError, IcebergTableIdentifier, Result};

mod orphan;
mod plan;
pub use orphan::{cleanup_orphan_objects, IcebergOrphanCleanupReport, IcebergOrphanCleanupRequest};
pub use plan::build_maintenance_plan;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergMaintenancePolicy {
    pub small_file_threshold_bytes: u64,
    pub target_file_size_bytes: u64,
    pub minimum_small_file_count: usize,
    pub snapshot_max_age_ms: i64,
    pub retain_last_snapshots: usize,
    pub orphan_min_age_ms: i64,
}

impl Default for IcebergMaintenancePolicy {
    fn default() -> Self {
        Self {
            small_file_threshold_bytes: 32 * 1024 * 1024,
            target_file_size_bytes: 256 * 1024 * 1024,
            minimum_small_file_count: 5,
            snapshot_max_age_ms: 7 * 24 * 60 * 60 * 1_000,
            retain_last_snapshots: 10,
            orphan_min_age_ms: 24 * 60 * 60 * 1_000,
        }
    }
}

impl IcebergMaintenancePolicy {
    pub fn validate(&self) -> Result<()> {
        if self.small_file_threshold_bytes == 0
            || self.target_file_size_bytes <= self.small_file_threshold_bytes
            || self.minimum_small_file_count < 2
            || self.snapshot_max_age_ms <= 0
            || self.retain_last_snapshots == 0
            || self.orphan_min_age_ms <= 0
        {
            return Err(IcebergIntegrationError::UnsafeMaintenancePlan {
                target: "maintenance_policy".to_string(),
                reason: "thresholds must be positive, target size must exceed the small-file threshold, at least two files are required, and at least one snapshot must be retained"
                    .to_string(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergMaintenanceFile {
    pub file_uri: String,
    pub file_size_in_bytes: u64,
    pub record_count: Option<u64>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergMaintenancePlanStatus {
    NoCompactionNeeded,
    CompactionRequired,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergTableMaintenancePlan {
    pub target: IcebergTableIdentifier,
    pub planned_snapshot_id: i64,
    pub status: IcebergMaintenancePlanStatus,
    pub current_file_count: usize,
    pub small_file_count: usize,
    pub small_file_bytes: u64,
    pub target_file_size_bytes: u64,
    pub rewrite_data_files_sql: Option<String>,
    pub expire_snapshots_sql: String,
    pub remove_orphan_files_sql: String,
    pub execution_order: Vec<String>,
}

pub async fn plan_iceberg_table_maintenance(
    table: &apache_iceberg::table::Table,
    spark_catalog: &str,
    target: &IcebergTableIdentifier,
    policy: &IcebergMaintenancePolicy,
    now_ms: i64,
    pending_file_uris: &BTreeSet<String>,
) -> Result<IcebergTableMaintenancePlan> {
    policy.validate()?;
    if spark_catalog.trim().is_empty() || spark_catalog.trim() != spark_catalog {
        return unsafe_plan(target, "Spark catalog name must be clean and non-blank");
    }
    if !pending_file_uris.is_empty() {
        return unsafe_plan(
            target,
            &format!(
                "{} object(s) are protected by pending commit intents; reconcile the writer before maintenance",
                pending_file_uris.len()
            ),
        );
    }
    let snapshot = table.metadata().current_snapshot().ok_or_else(|| {
        IcebergIntegrationError::UnsafeMaintenancePlan {
            target: target.qualified_name(),
            reason: "table has no current snapshot to bind the maintenance plan".to_string(),
        }
    })?;
    let scan = table
        .scan()
        .build()
        .map_err(|error| maintenance_catalog_error(target, "build_maintenance_scan", error))?;
    let mut stream = scan
        .plan_files()
        .await
        .map_err(|error| maintenance_catalog_error(target, "plan_maintenance_files", error))?;
    let mut files = Vec::new();
    while let Some(task) = stream
        .try_next()
        .await
        .map_err(|error| maintenance_catalog_error(target, "read_maintenance_files", error))?
    {
        files.push(IcebergMaintenanceFile {
            file_uri: task.data_file_path,
            file_size_in_bytes: task.file_size_in_bytes,
            record_count: task.record_count,
        });
    }
    build_maintenance_plan(
        target,
        spark_catalog,
        snapshot.snapshot_id(),
        &files,
        policy,
        now_ms,
    )
}

fn unsafe_plan<T>(target: &IcebergTableIdentifier, reason: &str) -> Result<T> {
    Err(IcebergIntegrationError::UnsafeMaintenancePlan {
        target: target.qualified_name(),
        reason: reason.to_string(),
    })
}

fn maintenance_catalog_error(
    target: &IcebergTableIdentifier,
    operation: &'static str,
    error: impl std::fmt::Display,
) -> IcebergIntegrationError {
    IcebergIntegrationError::Catalog {
        operation,
        target: target.qualified_name(),
        message: error.to_string(),
    }
}
