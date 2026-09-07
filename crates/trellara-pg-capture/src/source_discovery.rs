use tokio_postgres::Client;

use crate::{
    attach_postgres_risk_evidence_to_slots, inspect_tables,
    source_slot_inspection::observed_replication_slot_status_from_row, ReplicationSlotStatus,
    Result, TablePreflight, TableSelector,
};

pub(crate) const DISCOVER_USER_TABLES_SQL: &str = r#"
            select n.nspname::text as schema_name,
                   c.relname::text as table_name
              from pg_class c
              join pg_namespace n
                on n.oid = c.relnamespace
             where c.relkind in ('r', 'p')
               and n.nspname not like 'pg_%'
               and n.nspname <> 'information_schema'
             order by n.nspname, c.relname
            "#;

pub(crate) const INSPECT_LOGICAL_REPLICATION_SLOTS_SQL: &str = r#"
            select s.slot_name,
                   s.plugin,
                   s.active,
                   nullif(to_jsonb(s)->>'failover', '')::boolean as failover,
                   nullif(to_jsonb(s)->>'synced', '')::boolean as synced,
                   to_jsonb(s)->>'inactive_since' as inactive_since,
                   nullif(current_setting('idle_replication_slot_timeout', true), '') as idle_replication_slot_timeout,
                   s.restart_lsn::text as restart_lsn,
                   s.confirmed_flush_lsn::text as confirmed_flush_lsn,
                   case
                       when s.restart_lsn is null then null
                       else pg_wal_lsn_diff(pg_current_wal_lsn(), s.restart_lsn)::bigint
                   end as retained_wal_bytes,
                   to_jsonb(s)->>'wal_status' as wal_status,
                   nullif(to_jsonb(s)->>'safe_wal_size', '')::bigint as safe_wal_size_bytes,
                   to_jsonb(s)->>'invalidation_reason' as invalidation_reason
              from pg_replication_slots s
             where s.slot_type = 'logical'
             order by s.slot_name
            "#;

pub(crate) async fn discover_user_tables(client: &Client) -> Result<Vec<TableSelector>> {
    let rows = client.query(DISCOVER_USER_TABLES_SQL, &[]).await?;
    Ok(rows
        .into_iter()
        .map(|row| TableSelector {
            schema: row.get("schema_name"),
            name: row.get("table_name"),
        })
        .collect())
}

pub(crate) async fn inspect_all_user_tables(client: &Client) -> Result<Vec<TablePreflight>> {
    let tables = discover_user_tables(client).await?;
    inspect_tables(client, &tables).await
}

pub(crate) async fn inspect_logical_replication_slots(
    client: &Client,
) -> Result<Vec<ReplicationSlotStatus>> {
    let rows = client
        .query(INSPECT_LOGICAL_REPLICATION_SLOTS_SQL, &[])
        .await?;
    let mut slots = rows
        .into_iter()
        .map(observed_replication_slot_status_from_row)
        .collect::<Vec<_>>();
    attach_postgres_risk_evidence_to_slots(client, &mut slots).await;
    Ok(slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_queries_are_read_only_and_skip_system_schemas() {
        assert!(DISCOVER_USER_TABLES_SQL.contains("pg_class"));
        assert!(DISCOVER_USER_TABLES_SQL.contains("information_schema"));
        assert!(INSPECT_LOGICAL_REPLICATION_SLOTS_SQL.contains("slot_type = 'logical'"));
        assert!(!INSPECT_LOGICAL_REPLICATION_SLOTS_SQL.contains("create "));
    }
}
