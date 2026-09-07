use time::macros::format_description;
use time::OffsetDateTime;

use super::{
    IcebergMaintenanceFile, IcebergMaintenancePlanStatus, IcebergMaintenancePolicy,
    IcebergTableMaintenancePlan,
};
use crate::{IcebergIntegrationError, IcebergTableIdentifier, Result};

pub fn build_maintenance_plan(
    target: &IcebergTableIdentifier,
    spark_catalog: &str,
    planned_snapshot_id: i64,
    files: &[IcebergMaintenanceFile],
    policy: &IcebergMaintenancePolicy,
    now_ms: i64,
) -> Result<IcebergTableMaintenancePlan> {
    policy.validate()?;
    if planned_snapshot_id <= 0 {
        return Err(IcebergIntegrationError::UnsafeMaintenancePlan {
            target: target.qualified_name(),
            reason: "planned snapshot id must be positive".to_string(),
        });
    }
    let small_files = files
        .iter()
        .filter(|file| file.file_size_in_bytes < policy.small_file_threshold_bytes)
        .collect::<Vec<_>>();
    let small_file_bytes = small_files
        .iter()
        .try_fold(0u64, |total, file| {
            total.checked_add(file.file_size_in_bytes)
        })
        .ok_or(IcebergIntegrationError::CountOverflow {
            field: "maintenance.small_file_bytes",
        })?;
    let needs_compaction = small_files.len() >= policy.minimum_small_file_count;
    let procedure_catalog = quote_identifier(spark_catalog);
    let table_literal = sql_literal(&target.qualified_name());
    let rewrite_data_files_sql = needs_compaction.then(|| {
        format!(
            "CALL {procedure_catalog}.system.rewrite_data_files(table => {table_literal}, strategy => 'binpack', options => map('target-file-size-bytes', '{}', 'min-input-files', '{}'))",
            policy.target_file_size_bytes, policy.minimum_small_file_count
        )
    });
    let snapshot_cutoff = cutoff(target, now_ms, policy.snapshot_max_age_ms, "snapshot")?;
    let orphan_cutoff = cutoff(target, now_ms, policy.orphan_min_age_ms, "orphan")?;
    let expire_snapshots_sql = format!(
        "CALL {procedure_catalog}.system.expire_snapshots(table => {table_literal}, older_than => TIMESTAMP {}, retain_last => {})",
        sql_literal(&spark_timestamp(snapshot_cutoff)?),
        policy.retain_last_snapshots
    );
    let remove_orphan_files_sql = format!(
        "CALL {procedure_catalog}.system.remove_orphan_files(table => {table_literal}, older_than => TIMESTAMP {})",
        sql_literal(&spark_timestamp(orphan_cutoff)?)
    );
    let mut execution_order = Vec::new();
    if rewrite_data_files_sql.is_some() {
        execution_order.push("rewrite_data_files".to_string());
    }
    execution_order.extend([
        "expire_snapshots".to_string(),
        "remove_orphan_files".to_string(),
    ]);
    Ok(IcebergTableMaintenancePlan {
        target: target.clone(),
        planned_snapshot_id,
        status: if needs_compaction {
            IcebergMaintenancePlanStatus::CompactionRequired
        } else {
            IcebergMaintenancePlanStatus::NoCompactionNeeded
        },
        current_file_count: files.len(),
        small_file_count: small_files.len(),
        small_file_bytes,
        target_file_size_bytes: policy.target_file_size_bytes,
        rewrite_data_files_sql,
        expire_snapshots_sql,
        remove_orphan_files_sql,
        execution_order,
    })
}

fn cutoff(target: &IcebergTableIdentifier, now_ms: i64, age_ms: i64, kind: &str) -> Result<i64> {
    now_ms
        .checked_sub(age_ms)
        .ok_or_else(|| IcebergIntegrationError::UnsafeMaintenancePlan {
            target: target.qualified_name(),
            reason: format!("{kind} cutoff timestamp overflowed"),
        })
}

fn spark_timestamp(timestamp_ms: i64) -> Result<String> {
    let seconds = timestamp_ms.div_euclid(1_000);
    let nanos = u32::try_from(timestamp_ms.rem_euclid(1_000) * 1_000_000).map_err(|_| {
        IcebergIntegrationError::UnsafeMaintenancePlan {
            target: "timestamp".to_string(),
            reason: "timestamp nanoseconds overflowed".to_string(),
        }
    })?;
    OffsetDateTime::from_unix_timestamp(seconds)
        .and_then(|time| time.replace_nanosecond(nanos))
        .map_err(timestamp_error)?
        .format(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
        ))
        .map_err(timestamp_error)
}

fn timestamp_error(error: impl std::fmt::Display) -> IcebergIntegrationError {
    IcebergIntegrationError::UnsafeMaintenancePlan {
        target: "timestamp".to_string(),
        reason: error.to_string(),
    }
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
