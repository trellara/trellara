pub(crate) const UPSERT_SNAPSHOT_RUN: &str = r#"
    insert into trellara.snapshot_runs
        (source_id, dataset_id, run_id, state, slot_name, consistent_lsn,
         current_relation, copied_rows, failure_reason)
    values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    on conflict (source_id, dataset_id, run_id) do update
        set state = excluded.state,
            slot_name = excluded.slot_name,
            consistent_lsn = excluded.consistent_lsn,
            current_relation = excluded.current_relation,
            copied_rows = excluded.copied_rows,
            failure_reason = excluded.failure_reason,
            updated_at = now()
    "#;

pub(crate) const LOAD_SNAPSHOT_RUN: &str = r#"
    select source_id,
           dataset_id,
           run_id,
           state,
           slot_name,
           consistent_lsn,
           current_relation,
           copied_rows,
           failure_reason,
           started_at::text,
           updated_at::text
      from trellara.snapshot_runs
     where source_id = $1
       and dataset_id = $2
       and run_id = $3
    "#;

pub(crate) const LOAD_LATEST_SNAPSHOT_RUN: &str = r#"
    select source_id,
           dataset_id,
           run_id,
           state,
           slot_name,
           consistent_lsn,
           current_relation,
           copied_rows,
           failure_reason,
           started_at::text,
           updated_at::text
      from trellara.snapshot_runs
     where source_id = $1
       and dataset_id = $2
     order by updated_at desc
     limit 1
    "#;

pub(crate) const UPSERT_SNAPSHOT_TABLE_PROGRESS: &str = r#"
    insert into trellara.snapshot_table_progress
        (source_id, dataset_id, run_id, relation, state, copied_rows, watermark_lsn)
    values ($1, $2, $3, $4, $5, $6, $7)
    on conflict (source_id, dataset_id, run_id, relation) do update
        set state = excluded.state,
            copied_rows = excluded.copied_rows,
            watermark_lsn = excluded.watermark_lsn,
            updated_at = now()
    "#;

pub(crate) const LOAD_SNAPSHOT_TABLE_PROGRESS: &str = r#"
    select source_id,
           dataset_id,
           run_id,
           relation,
           state,
           copied_rows,
           watermark_lsn,
           updated_at::text
      from trellara.snapshot_table_progress
     where source_id = $1
       and dataset_id = $2
       and run_id = $3
       and relation = $4
    "#;

pub(crate) const LIST_SNAPSHOT_TABLE_PROGRESS: &str = r#"
    select source_id,
           dataset_id,
           run_id,
           relation,
           state,
           copied_rows,
           watermark_lsn,
           updated_at::text
      from trellara.snapshot_table_progress
     where source_id = $1
       and dataset_id = $2
       and run_id = $3
     order by relation
    "#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_run_upsert_preserves_flow_run_idempotency_key() {
        assert!(UPSERT_SNAPSHOT_RUN.contains("trellara.snapshot_runs"));
        assert!(UPSERT_SNAPSHOT_RUN.contains("on conflict (source_id, dataset_id, run_id)"));
        assert!(UPSERT_SNAPSHOT_RUN.contains("updated_at = now()"));
    }

    #[test]
    fn latest_snapshot_run_query_orders_by_recent_update() {
        assert!(LOAD_LATEST_SNAPSHOT_RUN.contains("order by updated_at desc"));
        assert!(LOAD_LATEST_SNAPSHOT_RUN.contains("limit 1"));
    }

    #[test]
    fn table_progress_upsert_preserves_relation_idempotency_key() {
        assert!(UPSERT_SNAPSHOT_TABLE_PROGRESS
            .contains("on conflict (source_id, dataset_id, run_id, relation)"));
        assert!(LIST_SNAPSHOT_TABLE_PROGRESS.contains("order by relation"));
    }
}
