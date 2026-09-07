pub(crate) const UPSERT_DDL_BARRIER: &str = r#"
    insert into trellara.ddl_barriers
        (source_id, database_id, dataset_id, barrier_id, barrier_lsn, schema_version,
         cdc_transaction_boundary, required_sinks, requires_global_partition_pause)
    values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    on conflict (source_id, database_id, dataset_id, barrier_id) do update
        set barrier_lsn = excluded.barrier_lsn,
            schema_version = excluded.schema_version,
            cdc_transaction_boundary = excluded.cdc_transaction_boundary,
            required_sinks = excluded.required_sinks,
            requires_global_partition_pause = excluded.requires_global_partition_pause,
            updated_at = now()
    "#;

pub(crate) const UPSERT_DDL_BARRIER_ACK: &str = r#"
    insert into trellara.ddl_barrier_acks
        (source_id, database_id, dataset_id, barrier_id, sink, ack_lsn, schema_version,
         accepted, detail)
    values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    on conflict (source_id, database_id, dataset_id, barrier_id, sink) do update
        set ack_lsn = excluded.ack_lsn,
            schema_version = excluded.schema_version,
            accepted = excluded.accepted,
            detail = excluded.detail,
            acked_at = now()
      where excluded.ack_lsn::pg_lsn >= trellara.ddl_barrier_acks.ack_lsn::pg_lsn
    "#;

pub(crate) const LOAD_DDL_BARRIER: &str = r#"
    select source_id,
           database_id,
           dataset_id,
           barrier_id,
           barrier_lsn,
           schema_version,
           cdc_transaction_boundary,
           required_sinks,
           requires_global_partition_pause
      from trellara.ddl_barriers
     where source_id = $1
       and database_id = $2
       and dataset_id = $3
       and barrier_id = $4
    "#;

pub(crate) const LOAD_DDL_BARRIER_ACKS: &str = r#"
    select source_id,
           database_id,
           dataset_id,
           barrier_id,
           sink,
           ack_lsn,
           schema_version,
           accepted,
           detail
      from trellara.ddl_barrier_acks
     where source_id = $1
       and database_id = $2
       and dataset_id = $3
       and barrier_id = $4
    "#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ddl_barrier_upsert_uses_barrier_idempotency_key() {
        assert!(UPSERT_DDL_BARRIER.contains("trellara.ddl_barriers"));
        assert!(UPSERT_DDL_BARRIER
            .contains("on conflict (source_id, database_id, dataset_id, barrier_id)"));
        assert!(UPSERT_DDL_BARRIER.contains("cdc_transaction_boundary"));
        assert!(UPSERT_DDL_BARRIER.contains("requires_global_partition_pause"));
    }

    #[test]
    fn ddl_barrier_ack_upsert_uses_sink_scoped_idempotency_key() {
        assert!(UPSERT_DDL_BARRIER_ACK.contains("trellara.ddl_barrier_acks"));
        assert!(UPSERT_DDL_BARRIER_ACK
            .contains("on conflict (source_id, database_id, dataset_id, barrier_id, sink)"));
        assert!(UPSERT_DDL_BARRIER_ACK
            .contains("excluded.ack_lsn::pg_lsn >= trellara.ddl_barrier_acks.ack_lsn::pg_lsn"));
        assert!(UPSERT_DDL_BARRIER_ACK.contains("acked_at = now()"));
    }

    #[test]
    fn ddl_barrier_summary_queries_load_barrier_and_ack_set() {
        assert!(LOAD_DDL_BARRIER.contains("from trellara.ddl_barriers"));
        assert!(LOAD_DDL_BARRIER_ACKS.contains("from trellara.ddl_barrier_acks"));
        assert!(LOAD_DDL_BARRIER_ACKS.contains("and barrier_id = $4"));
    }
}
