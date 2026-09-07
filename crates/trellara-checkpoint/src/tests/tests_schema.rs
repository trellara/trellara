use super::*;

#[test]
fn postgres_schema_contains_dedup_primary_key() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("trellara.applied_transactions"));
    assert!(sql
        .contains("primary key (source_id, database_id, dataset_id, transaction_id, commit_lsn)"));
    assert!(sql.contains("add column if not exists database_id"));
    assert!(sql.contains("drop constraint if exists applied_transactions_pkey"));
}

#[test]
fn postgres_schema_enforces_flow_checkpoint_lsn_ordering() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("flow_checkpoints_durable_not_ahead_of_seen_check"));
    assert!(sql.contains("flow_checkpoints_applied_not_ahead_of_durable_check"));
    assert!(sql.contains("last_durable_lsn::pg_lsn <= last_seen_lsn::pg_lsn"));
    assert!(sql.contains("last_applied_lsn::pg_lsn <= last_durable_lsn::pg_lsn"));
}

#[test]
fn postgres_schema_contains_apply_quarantine() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("trellara.apply_quarantine"));
    assert!(sql.contains("attempt_count bigint not null default 1"));
    assert!(sql
        .contains("primary key (source_id, database_id, dataset_id, transaction_id, commit_lsn)"));
    assert!(sql.contains("drop constraint if exists apply_quarantine_pkey"));
}

#[test]
fn postgres_schema_contains_reseed_events() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("trellara.reseed_events"));
    assert!(sql.contains("reseed_events_flow_completed_at_idx"));
}

#[test]
fn postgres_schema_contains_snapshot_handoff_events() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("trellara.snapshot_handoff_events"));
    assert!(sql.contains("snapshot_handoff_events_flow_completed_at_idx"));
}

#[test]
fn postgres_schema_contains_snapshot_runs_and_table_progress() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("trellara.snapshot_runs"));
    assert!(sql.contains("primary key (source_id, dataset_id, run_id)"));
    assert!(sql.contains("snapshot_runs_flow_updated_at_idx"));
    assert!(sql.contains("trellara.snapshot_table_progress"));
    assert!(sql.contains("primary key (source_id, dataset_id, run_id, relation)"));
    assert!(sql.contains("snapshot_table_progress_run_idx"));
    assert!(sql.contains("'stream_handoff_ready'"));
    assert!(sql.contains("check (copied_rows >= 0)"));
}

#[test]
fn postgres_schema_contains_validation_events() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("trellara.validation_events"));
    assert!(sql.contains("drift_relations text[] not null default '{}'"));
    assert!(sql.contains("add column if not exists drift_relations"));
    assert!(sql.contains("add column if not exists evidence_sha256"));
    assert!(sql.contains("validation_events_flow_completed_at_idx"));
}

#[test]
fn postgres_schema_contains_partition_checkpoints() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("trellara.partition_checkpoints"));
    assert!(sql.contains("primary key (source_id, dataset_id, partition_id)"));
    assert!(sql.contains("check (partition_id >= 0)"));
    assert!(sql.contains("partition_checkpoints_applied_not_ahead_of_durable_check"));
    assert!(sql.contains("last_applied_lsn::pg_lsn <= last_durable_lsn::pg_lsn"));
}

#[test]
fn postgres_schema_contains_ddl_barrier_ack_tables() {
    let sql = postgres_checkpoint_schema_sql();
    assert!(sql.contains("trellara.ddl_barriers"));
    assert!(sql.contains("cdc_transaction_boundary text not null"));
    assert!(sql.contains("add column if not exists cdc_transaction_boundary"));
    assert!(sql.contains("required_sinks text[] not null"));
    assert!(sql.contains("requires_global_partition_pause boolean not null default false"));
    assert!(sql.contains("or 'partition_visibility' = any(required_sinks)"));
    assert!(sql.contains("add column if not exists database_id"));
    assert!(sql.contains("primary key (source_id, database_id, dataset_id, barrier_id)"));
    assert!(sql.contains("drop constraint if exists ddl_barriers_pkey"));
    assert!(sql.contains("trellara.ddl_barrier_acks"));
    assert!(sql.contains("primary key (source_id, database_id, dataset_id, barrier_id, sink)"));
    assert!(sql.contains("drop constraint if exists ddl_barrier_acks_pkey"));
    assert!(sql.contains("check (sink = btrim(sink) and sink <> '')"));
    assert!(sql.contains("check (schema_version = btrim(schema_version) and schema_version <> '')"));
    assert!(sql.contains("check (btrim(detail) <> '')"));
    assert!(sql.contains("ddl_barrier_acks_flow_barrier_idx"));
}
