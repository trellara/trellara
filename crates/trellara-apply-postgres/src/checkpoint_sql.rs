pub(crate) const TRANSACTION_ALREADY_APPLIED: &str = r#"
select 1
  from trellara.applied_transactions
 where source_id = $1
   and database_id = $2
   and dataset_id = $3
   and transaction_id = $4
   and commit_lsn = $5
"#;

// Keep this as a plain insert. A duplicate key must roll back the surrounding
// target-DML transaction instead of letting a concurrent replay commit twice.
pub(crate) const RECORD_APPLIED_TRANSACTION: &str = r#"
insert into trellara.applied_transactions
    (source_id, database_id, dataset_id, transaction_id, commit_lsn)
values ($1, $2, $3, $4, $5)
"#;

pub(crate) const CLEAR_QUARANTINED_TRANSACTION: &str = r#"
delete from trellara.apply_quarantine
 where source_id = $1
   and database_id = $2
   and dataset_id = $3
   and transaction_id = $4
   and commit_lsn = $5
"#;

pub(crate) const UPSERT_FLOW_CHECKPOINT: &str = r#"
insert into trellara.flow_checkpoints
    (source_id, dataset_id, last_seen_lsn, last_durable_lsn, last_applied_lsn)
values ($1, $2, $3, $3, $3)
on conflict (source_id, dataset_id) do update
    set last_seen_lsn =
            case
                when excluded.last_seen_lsn = ''
                then trellara.flow_checkpoints.last_seen_lsn
                when trellara.flow_checkpoints.last_seen_lsn = ''
                  or excluded.last_seen_lsn::pg_lsn >= trellara.flow_checkpoints.last_seen_lsn::pg_lsn
                then excluded.last_seen_lsn
                else trellara.flow_checkpoints.last_seen_lsn
            end,
        last_durable_lsn =
            case
                when excluded.last_durable_lsn = ''
                then trellara.flow_checkpoints.last_durable_lsn
                when trellara.flow_checkpoints.last_durable_lsn = ''
                  or excluded.last_durable_lsn::pg_lsn >= trellara.flow_checkpoints.last_durable_lsn::pg_lsn
                then excluded.last_durable_lsn
                else trellara.flow_checkpoints.last_durable_lsn
            end,
        last_applied_lsn =
            case
                when excluded.last_applied_lsn = ''
                then trellara.flow_checkpoints.last_applied_lsn
                when trellara.flow_checkpoints.last_applied_lsn = ''
                  or excluded.last_applied_lsn::pg_lsn >= trellara.flow_checkpoints.last_applied_lsn::pg_lsn
                then excluded.last_applied_lsn
                else trellara.flow_checkpoints.last_applied_lsn
            end,
        updated_at = now()
"#;

pub(crate) const UPSERT_PARTITION_CHECKPOINT: &str = r#"
insert into trellara.partition_checkpoints
    (source_id, dataset_id, partition_id, last_durable_lsn, last_applied_lsn)
values ($1, $2, $3, $4, $4)
on conflict (source_id, dataset_id, partition_id) do update
    set last_durable_lsn =
            case
                when excluded.last_durable_lsn = ''
                then trellara.partition_checkpoints.last_durable_lsn
                when trellara.partition_checkpoints.last_durable_lsn = ''
                  or excluded.last_durable_lsn::pg_lsn >= trellara.partition_checkpoints.last_durable_lsn::pg_lsn
                then excluded.last_durable_lsn
                else trellara.partition_checkpoints.last_durable_lsn
            end,
        last_applied_lsn =
            case
                when excluded.last_applied_lsn = ''
                then trellara.partition_checkpoints.last_applied_lsn
                when trellara.partition_checkpoints.last_applied_lsn = ''
                  or excluded.last_applied_lsn::pg_lsn >= trellara.partition_checkpoints.last_applied_lsn::pg_lsn
                then excluded.last_applied_lsn
                else trellara.partition_checkpoints.last_applied_lsn
            end,
        updated_at = now()
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applied_transaction_insert_uses_full_transaction_identity() {
        for column in [
            "source_id",
            "database_id",
            "dataset_id",
            "transaction_id",
            "commit_lsn",
        ] {
            assert!(TRANSACTION_ALREADY_APPLIED.contains(column));
            assert!(RECORD_APPLIED_TRANSACTION.contains(column));
            assert!(CLEAR_QUARANTINED_TRANSACTION.contains(column));
        }
    }

    #[test]
    fn applied_transaction_insert_fails_closed_on_duplicate_key() {
        assert!(!RECORD_APPLIED_TRANSACTION
            .to_lowercase()
            .contains("on conflict"));
    }

    #[test]
    fn checkpoint_upserts_keep_lsn_boundaries_monotonic() {
        for column in ["last_seen_lsn", "last_durable_lsn", "last_applied_lsn"] {
            let comparison =
                format!("excluded.{column}::pg_lsn >= trellara.flow_checkpoints.{column}::pg_lsn");
            assert!(UPSERT_FLOW_CHECKPOINT.contains(&comparison));
        }
        for column in ["last_durable_lsn", "last_applied_lsn"] {
            let comparison = format!(
                "excluded.{column}::pg_lsn >= trellara.partition_checkpoints.{column}::pg_lsn"
            );
            assert!(UPSERT_PARTITION_CHECKPOINT.contains(&comparison));
        }
    }
}
