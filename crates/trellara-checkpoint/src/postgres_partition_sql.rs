pub(crate) const UPSERT_PARTITION_CHECKPOINT: &str = r#"
    insert into trellara.partition_checkpoints
        (source_id, dataset_id, partition_id, last_durable_lsn, last_applied_lsn)
    values ($1, $2, $3, $4, $5)
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

pub(crate) const LOAD_PARTITION_CHECKPOINTS: &str = r#"
    select source_id,
           dataset_id,
           partition_id,
           last_durable_lsn,
           last_applied_lsn
      from trellara.partition_checkpoints
     where source_id = $1
       and dataset_id = $2
     order by partition_id
    "#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_checkpoint_upsert_uses_partition_idempotency_key() {
        assert!(UPSERT_PARTITION_CHECKPOINT.contains("trellara.partition_checkpoints"));
        assert!(UPSERT_PARTITION_CHECKPOINT
            .contains("on conflict (source_id, dataset_id, partition_id)"));
    }

    #[test]
    fn partition_checkpoint_upsert_keeps_lsn_progress_monotonic() {
        assert!(UPSERT_PARTITION_CHECKPOINT
            .contains("excluded.last_durable_lsn::pg_lsn >= trellara.partition_checkpoints.last_durable_lsn::pg_lsn"));
        assert!(UPSERT_PARTITION_CHECKPOINT
            .contains("excluded.last_applied_lsn::pg_lsn >= trellara.partition_checkpoints.last_applied_lsn::pg_lsn"));
        assert!(UPSERT_PARTITION_CHECKPOINT.contains("when excluded.last_durable_lsn = ''"));
        assert!(UPSERT_PARTITION_CHECKPOINT.contains("when excluded.last_applied_lsn = ''"));
    }

    #[test]
    fn partition_checkpoint_load_returns_stable_partition_order() {
        assert!(LOAD_PARTITION_CHECKPOINTS.contains("order by partition_id"));
    }
}
