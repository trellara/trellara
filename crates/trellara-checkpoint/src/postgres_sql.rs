pub(crate) const LOAD_CHECKPOINT: &str = r#"
    select source_id,
           dataset_id,
           last_seen_lsn,
           last_durable_lsn,
           last_applied_lsn
      from trellara.flow_checkpoints
     where source_id = $1
       and dataset_id = $2
    "#;

pub(crate) const UPSERT_CHECKPOINT: &str = r#"
    insert into trellara.flow_checkpoints
        (source_id, dataset_id, last_seen_lsn, last_durable_lsn, last_applied_lsn)
    values ($1, $2, $3, $4, $5)
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

pub(crate) const APPLIED_TRANSACTION_EXISTS: &str = r#"
    select 1
      from trellara.applied_transactions
     where source_id = $1
       and database_id = $2
       and dataset_id = $3
       and transaction_id = $4
       and commit_lsn = $5
    "#;

pub(crate) const INSERT_APPLIED_TRANSACTION: &str = r#"
    insert into trellara.applied_transactions
        (source_id, database_id, dataset_id, transaction_id, commit_lsn)
    values ($1, $2, $3, $4, $5)
    on conflict (source_id, database_id, dataset_id, transaction_id, commit_lsn) do nothing
    "#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_upsert_uses_flow_idempotency_key() {
        assert!(UPSERT_CHECKPOINT.contains("trellara.flow_checkpoints"));
        assert!(UPSERT_CHECKPOINT.contains("on conflict (source_id, dataset_id)"));
    }

    #[test]
    fn checkpoint_upsert_keeps_all_lsn_boundaries_monotonic() {
        for column in ["last_seen_lsn", "last_durable_lsn", "last_applied_lsn"] {
            let comparison =
                format!("excluded.{column}::pg_lsn >= trellara.flow_checkpoints.{column}::pg_lsn");
            let empty_guard = format!("when excluded.{column} = ''");

            assert!(UPSERT_CHECKPOINT.contains(&comparison));
            assert!(UPSERT_CHECKPOINT.contains(&empty_guard));
        }
    }

    #[test]
    fn applied_transaction_queries_use_full_transaction_identity() {
        for query in [APPLIED_TRANSACTION_EXISTS, INSERT_APPLIED_TRANSACTION] {
            assert!(query.contains("source_id"));
            assert!(query.contains("database_id"));
            assert!(query.contains("dataset_id"));
            assert!(query.contains("transaction_id"));
            assert!(query.contains("commit_lsn"));
        }
        assert!(INSERT_APPLIED_TRANSACTION.contains(
            "on conflict (source_id, database_id, dataset_id, transaction_id, commit_lsn)"
        ));
    }
}
