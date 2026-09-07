pub(crate) const INSERT_INTENT: &str = r#"
insert into trellara.iceberg_commit_intents (
    dataset_id, epoch_id, epoch_commit_id, lake_table_name, relation, target,
    table_commit_id, manifest_digest, file_count, record_count, planned_at
) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
on conflict (dataset_id, epoch_id, target, table_commit_id) do nothing
"#;

pub(crate) const LOAD_INTENT: &str = r#"
select dataset_id, epoch_id, epoch_commit_id, lake_table_name, relation, target,
       table_commit_id, manifest_digest, file_count, record_count, planned_at
  from trellara.iceberg_commit_intents
 where dataset_id = $1 and epoch_id = $2 and target = $3 and table_commit_id = $4
"#;

pub(crate) const INSERT_RECEIPT: &str = r#"
insert into trellara.iceberg_commit_receipts (
    dataset_id, epoch_id, epoch_commit_id, target, table_commit_id, snapshot_id,
    file_count, record_count, status, committed_at
) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
on conflict (dataset_id, epoch_id, target, table_commit_id) do nothing
"#;

pub(crate) const LOAD_RECEIPT: &str = r#"
select dataset_id, epoch_id, epoch_commit_id, target, table_commit_id, snapshot_id,
       file_count, record_count, status, committed_at
  from trellara.iceberg_commit_receipts
 where dataset_id = $1 and epoch_id = $2 and target = $3 and table_commit_id = $4
"#;

pub(crate) const LIST_RECEIPTS_FOR_EPOCH: &str = r#"
select dataset_id, epoch_id, epoch_commit_id, target, table_commit_id, snapshot_id,
       file_count, record_count, status, committed_at
  from trellara.iceberg_commit_receipts
 where dataset_id = $1 and epoch_id = $2
 order by target, table_commit_id
"#;
