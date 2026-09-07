#[cfg(test)]
pub(crate) const QUARANTINE_SELECT_COLUMNS: &str = r#"
    source_id,
    database_id,
    dataset_id,
    transaction_id,
    commit_lsn,
    reason,
    detail,
    attempt_count,
    last_seen_at::text
    "#;

pub(crate) const LOAD_LATEST_QUARANTINE: &str = r#"
    select source_id,
           database_id,
           dataset_id,
           transaction_id,
           commit_lsn,
           reason,
           detail,
           attempt_count,
           last_seen_at::text
     from trellara.apply_quarantine
     where source_id = $1
       and dataset_id = $2
     order by last_seen_at desc
     limit 1
    "#;

pub(crate) const LOAD_QUARANTINE: &str = r#"
    select source_id,
           database_id,
           dataset_id,
           transaction_id,
           commit_lsn,
           reason,
           detail,
           attempt_count,
           last_seen_at::text
     from trellara.apply_quarantine
     where source_id = $1
       and database_id = $2
       and dataset_id = $3
       and transaction_id = $4
       and commit_lsn = $5
    "#;

pub(crate) const LIST_QUARANTINE: &str = r#"
    select source_id,
           database_id,
           dataset_id,
           transaction_id,
           commit_lsn,
           reason,
           detail,
           attempt_count,
           last_seen_at::text
      from trellara.apply_quarantine
     where source_id = $1
       and dataset_id = $2
     order by last_seen_at desc
     limit $3
    "#;

pub(crate) const CLEAR_QUARANTINE: &str = r#"
    delete from trellara.apply_quarantine
     where source_id = $1
       and database_id = $2
       and dataset_id = $3
       and transaction_id = $4
       and commit_lsn = $5
    "#;

pub(crate) const CLEAR_APPLIED_TRANSACTION: &str = r#"
    delete from trellara.applied_transactions
     where source_id = $1
       and database_id = $2
       and dataset_id = $3
       and transaction_id = $4
       and commit_lsn = $5
    "#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarantine_load_queries_use_transaction_boundary_identity() {
        for query in [LOAD_QUARANTINE, CLEAR_QUARANTINE, CLEAR_APPLIED_TRANSACTION] {
            assert!(query.contains("source_id = $1"));
            assert!(query.contains("database_id = $2"));
            assert!(query.contains("dataset_id = $3"));
            assert!(query.contains("transaction_id = $4"));
            assert!(query.contains("commit_lsn = $5"));
        }
    }

    #[test]
    fn quarantine_latest_and_list_queries_order_by_last_seen_time() {
        assert!(LOAD_LATEST_QUARANTINE.contains("order by last_seen_at desc"));
        assert!(LOAD_LATEST_QUARANTINE.contains("limit 1"));
        assert!(LIST_QUARANTINE.contains("order by last_seen_at desc"));
        assert!(LIST_QUARANTINE.contains("limit $3"));
    }

    #[test]
    fn quarantine_select_columns_include_repair_evidence() {
        assert!(QUARANTINE_SELECT_COLUMNS.contains("reason"));
        assert!(QUARANTINE_SELECT_COLUMNS.contains("detail"));
        assert!(QUARANTINE_SELECT_COLUMNS.contains("attempt_count"));
    }
}
