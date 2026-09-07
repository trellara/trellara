use crate::{lake_table_prefix, non_empty_string, LakeDdlTable, TableConfig};

pub(crate) fn lake_raw_cdc_table(
    dataset_id: &str,
    table: &TableConfig,
    visibility_boundary: &str,
    uses_ownership_key: bool,
) -> LakeDdlTable {
    let verify = table.verify.clone().unwrap_or_default();
    let relation = table.relation_id().display_name();
    let table_prefix = lake_table_prefix(dataset_id, &table.schema, &table.name);
    let bucket_column = if uses_ownership_key {
        "__trellara_partition_key"
    } else {
        "__trellara_source_id"
    };

    LakeDdlTable {
        relation,
        materialization: "raw_cdc_append_only".to_string(),
        table_name: format!("{table_prefix}__raw_cdc"),
        primary_key: non_empty_string(verify.primary_key),
        checkpoint_column: "__trellara_commit_lsn".to_string(),
        visibility_boundary: visibility_boundary.to_string(),
        partitioning: vec![
            format!("bucket(32, {bucket_column})"),
            "days(__trellara_commit_timestamp)".to_string(),
            "__trellara_relation".to_string(),
        ],
        ddl: lake_raw_cdc_ddl(&table_prefix),
    }
}

fn lake_raw_cdc_ddl(table_prefix: &str) -> String {
    format!(
        "CREATE TABLE {table_prefix}__raw_cdc (\n  __trellara_idempotency_key TEXT NOT NULL,\n  __trellara_source_id TEXT NOT NULL,\n  __trellara_database_id TEXT NOT NULL,\n  __trellara_dataset_id TEXT NOT NULL,\n  __trellara_relation TEXT NOT NULL,\n  __trellara_transaction_id TEXT NOT NULL,\n  __trellara_begin_lsn TEXT,\n  __trellara_commit_lsn TEXT NOT NULL,\n  __trellara_commit_timestamp TIMESTAMP NOT NULL,\n  __trellara_total_order BIGINT NOT NULL,\n  __trellara_operation TEXT NOT NULL,\n  __trellara_record_key TEXT,\n  __trellara_schema_fingerprint BIGINT,\n  __trellara_schema_version BIGINT,\n  __trellara_ddl_barrier_id TEXT,\n  __trellara_ddl_release_gate TEXT,\n  __trellara_ddl_schema_fingerprint_before BIGINT,\n  __trellara_ddl_schema_fingerprint_after BIGINT,\n  __trellara_envelope_checksum BIGINT NOT NULL,\n  __trellara_manifest_id TEXT,\n  __trellara_manifest_boundary_mode TEXT,\n  __trellara_manifest_global_event_count BIGINT,\n  __trellara_manifest_participating_partition_count BIGINT,\n  __trellara_partition_key TEXT,\n  __trellara_epoch_id TEXT NOT NULL,\n  __trellara_ingested_at TIMESTAMP NOT NULL,\n  __trellara_payload_before TEXT,\n  __trellara_payload_after TEXT\n);"
    )
}

pub(crate) fn lake_epoch_metadata_ddl_tables(
    dataset_id: &str,
    visibility_boundary: &str,
) -> Vec<LakeDdlTable> {
    let prefix = lake_table_prefix(dataset_id, "trellara", "fanin");
    [
        (
            "epoch_metadata",
            "_trellara_epochs",
            "__trellara_epoch_id",
            "CREATE TABLE {table_name} (\n  epoch_id TEXT NOT NULL,\n  dataset_id TEXT NOT NULL,\n  state TEXT NOT NULL,\n  policy TEXT NOT NULL,\n  opened_at TIMESTAMP,\n  sealed_at TIMESTAMP,\n  required_source_count BIGINT NOT NULL,\n  complete_source_count BIGINT NOT NULL,\n  missing_source_count BIGINT NOT NULL,\n  quarantined_source_count BIGINT NOT NULL,\n  transaction_count BIGINT NOT NULL,\n  change_count BIGINT NOT NULL,\n  checksum_rollup BIGINT NOT NULL,\n  manifest_digest TEXT NOT NULL,\n  iceberg_snapshot_id TEXT NOT NULL,\n  raw_table_snapshot_ids_json TEXT NOT NULL\n);",
            vec!["unpartitioned_l1_metadata_release_marker".to_string()],
        ),
        (
            "epoch_source_metadata",
            "_trellara_epoch_sources",
            "__trellara_epoch_id",
            "CREATE TABLE {table_name} (\n  epoch_id TEXT NOT NULL,\n  source_id TEXT NOT NULL,\n  state TEXT NOT NULL,\n  start_lsn TEXT NOT NULL,\n  end_lsn TEXT NOT NULL,\n  transaction_count BIGINT NOT NULL,\n  change_count BIGINT NOT NULL,\n  checksum_rollup BIGINT NOT NULL,\n  lag_reason TEXT\n);",
            vec!["epoch_id".to_string(), "bucket(64, source_id)".to_string()],
        ),
        (
            "epoch_table_metadata",
            "_trellara_epoch_tables",
            "__trellara_epoch_id",
            "CREATE TABLE {table_name} (\n  epoch_id TEXT NOT NULL,\n  relation TEXT NOT NULL,\n  transaction_count BIGINT NOT NULL,\n  change_count BIGINT NOT NULL,\n  checksum_rollup BIGINT NOT NULL\n);",
            vec!["epoch_id".to_string(), "relation".to_string()],
        ),
        (
            "epoch_partition_metadata",
            "_trellara_epoch_partitions",
            "__trellara_epoch_id",
            "CREATE TABLE {table_name} (\n  epoch_id TEXT NOT NULL,\n  source_id TEXT NOT NULL,\n  partition_id BIGINT NOT NULL,\n  first_commit_lsn TEXT NOT NULL,\n  last_commit_lsn TEXT NOT NULL,\n  transaction_count BIGINT NOT NULL,\n  event_count BIGINT NOT NULL,\n  checksum_rollup BIGINT NOT NULL\n);",
            vec![
                "epoch_id".to_string(),
                "bucket(64, source_id)".to_string(),
                "partition_id".to_string(),
            ],
        ),
        (
            "epoch_quarantine_metadata",
            "_trellara_quarantine",
            "__trellara_epoch_id",
            "CREATE TABLE {table_name} (\n  epoch_id TEXT NOT NULL,\n  source_id TEXT,\n  transaction_id TEXT,\n  commit_lsn TEXT,\n  reason TEXT NOT NULL,\n  details TEXT,\n  recovery_command TEXT\n);",
            vec!["epoch_id".to_string(), "reason".to_string()],
        ),
        (
            "epoch_verification_metadata",
            "_trellara_verification",
            "__trellara_epoch_id",
            "CREATE TABLE {table_name} (\n  epoch_id TEXT NOT NULL,\n  verification_id TEXT NOT NULL,\n  input_transaction_count BIGINT NOT NULL,\n  input_change_count BIGINT NOT NULL,\n  lake_transaction_count BIGINT NOT NULL,\n  lake_change_count BIGINT NOT NULL,\n  checksum_status TEXT NOT NULL,\n  completed_at TIMESTAMP NOT NULL\n);",
            vec!["days(completed_at)".to_string(), "checksum_status".to_string()],
        ),
    ]
    .into_iter()
    .map(
        |(materialization, suffix, checkpoint_column, ddl_template, partitioning)| {
            let table_name = format!("{prefix}__{suffix}");
            LakeDdlTable {
                relation: "trellara.fanin".to_string(),
                materialization: materialization.to_string(),
                table_name: table_name.clone(),
                primary_key: None,
                checkpoint_column: checkpoint_column.to_string(),
                visibility_boundary: visibility_boundary.to_string(),
                partitioning,
                ddl: ddl_template.replace("{table_name}", &table_name),
            }
        },
    )
    .collect()
}
