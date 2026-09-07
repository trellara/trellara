use tokio_postgres::Client;

use crate::{ReplicationSlotStatus, Result, TablePreflight, TableSelector};

use crate::source_postgres_risk_inspection::attach_postgres_risk_evidence;
#[cfg(test)]
pub(crate) use crate::source_replica_identity_contract::replica_identity_contract_notes;
pub(crate) use crate::source_slot_inspection::replication_slot_status_from_row;
#[cfg(test)]
pub(crate) use crate::source_slot_inspection::slot_status_issues;
#[cfg(test)]
pub(crate) use crate::source_table_inspection::schema_fingerprint;
pub(crate) use crate::source_table_inspection::{
    fail_on_preflight_issues, table_preflight_from_row,
};

pub(crate) const INSPECT_TABLES_SQL: &str = r#"
            select selected.ordinality::int4 as ordinality,
                   selected.schema_name,
                   selected.table_name,
                   c.relreplident::text as relreplident,
                   coalesce(pk.primary_key_columns, array[]::text[]) as primary_key_columns,
                   coalesce(cols.ordinal_positions, array[]::int4[]) as column_ordinals,
                   coalesce(cols.column_names, array[]::text[]) as column_names,
                   coalesce(cols.type_oids, array[]::oid[]) as column_type_oids,
                   coalesce(cols.type_names, array[]::text[]) as column_type_names,
                   coalesce(cols.nullable, array[]::bool[]) as column_nullable
              from unnest($1::text[], $2::text[]) with ordinality
                   as selected(schema_name, table_name, ordinality)
              left join pg_namespace n
                     on n.nspname = selected.schema_name
              left join pg_class c
                     on c.relnamespace = n.oid
                    and c.relname = selected.table_name
                    and c.relkind in ('r', 'p')
              left join lateral (
                    select array_agg(a.attname order by pk.keys_ord) as primary_key_columns
                      from pg_index i
                      join lateral unnest(i.indkey) with ordinality as pk(attnum, keys_ord)
                        on true
                      join pg_attribute a
                        on a.attrelid = i.indrelid
                       and a.attnum = pk.attnum
                     where i.indrelid = c.oid
                       and i.indisprimary
              ) pk on true
              left join lateral (
                    select array_agg(a.attnum::int4 order by a.attnum) as ordinal_positions,
                           array_agg(a.attname order by a.attnum) as column_names,
                           array_agg(a.atttypid order by a.attnum) as type_oids,
                           array_agg(format_type(a.atttypid, a.atttypmod) order by a.attnum) as type_names,
                           array_agg(not a.attnotnull order by a.attnum) as nullable
                      from pg_attribute a
                     where a.attrelid = c.oid
                       and a.attnum > 0
                       and not a.attisdropped
              ) cols on true
             order by selected.ordinality
            "#;
pub(crate) const INSPECT_SLOT_STATUS_SQL: &str = r#"
            select selected.slot_name,
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
              from (select $1::name as slot_name) selected
              left join pg_replication_slots s
                     on s.slot_name = selected.slot_name
            "#;

pub(crate) async fn inspect_tables(
    client: &Client,
    tables: &[TableSelector],
) -> Result<Vec<TablePreflight>> {
    let rows = client
        .query(
            INSPECT_TABLES_SQL,
            &[
                &tables
                    .iter()
                    .map(|table| table.schema.clone())
                    .collect::<Vec<_>>(),
                &tables
                    .iter()
                    .map(|table| table.name.clone())
                    .collect::<Vec<_>>(),
            ],
        )
        .await?;

    Ok(rows.into_iter().map(table_preflight_from_row).collect())
}

pub(crate) async fn inspect_slot_status(
    client: &Client,
    slot_name: &str,
    expected_plugin: &str,
) -> Result<ReplicationSlotStatus> {
    let row = client
        .query_one(INSPECT_SLOT_STATUS_SQL, &[&slot_name])
        .await?;
    let mut status = replication_slot_status_from_row(row, expected_plugin);
    attach_postgres_risk_evidence(client, &mut status).await;
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_tables_query_preserves_native_postgres_type_oid_array() {
        assert!(INSPECT_TABLES_SQL.contains("array[]::oid[]"));
        assert!(INSPECT_TABLES_SQL.contains("array_agg(a.atttypid order by a.attnum)"));
        assert!(!INSPECT_TABLES_SQL.contains("a.atttypid::int4"));
    }

    #[test]
    fn inspect_slot_status_query_reads_retention_and_pg18_slot_metadata() {
        assert!(INSPECT_SLOT_STATUS_SQL.contains("pg_replication_slots"));
        assert!(INSPECT_SLOT_STATUS_SQL.contains("safe_wal_size"));
        assert!(INSPECT_SLOT_STATUS_SQL.contains("failover"));
        assert!(INSPECT_SLOT_STATUS_SQL.contains("idle_replication_slot_timeout"));
    }
}
