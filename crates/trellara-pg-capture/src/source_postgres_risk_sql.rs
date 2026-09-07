pub(crate) const WAL_RATE_SAMPLE_MS: u64 = 2_000;
pub(crate) const CURRENT_WAL_LSN_SQL: &str = "select pg_current_wal_lsn()::text as current_lsn";
pub(crate) const WAL_LSN_DIFF_SQL: &str =
    "select pg_wal_lsn_diff($1::pg_lsn, $2::pg_lsn)::bigint as wal_bytes";
pub(crate) const TXID_WRAPAROUND_SQL: &str = r#"
            select datname::text as oldest_database,
                   age(datfrozenxid)::bigint as oldest_datfrozenxid_age,
                   current_setting('autovacuum_freeze_max_age')::bigint as autovacuum_freeze_max_age
              from pg_database
             order by age(datfrozenxid) desc
             limit 1
            "#;
pub(crate) const SLOT_CATALOG_XMIN_SQL: &str = r#"
            select catalog_xmin::text as slot_catalog_xmin
              from pg_replication_slots
             where slot_name = $1
            "#;
pub(crate) const STALE_CATALOG_XMIN_AGE_TXIDS: i64 = 1_000_000;
pub(crate) const XMIN_HORIZON_SQL: &str = r#"
            select (select slot_name::text
                    from pg_replication_slots
                     where catalog_xmin is not null
                       and age(catalog_xmin)::bigint >= $1
                     order by age(catalog_xmin) desc
                     limit 1) as oldest_catalog_xmin_slot,
                   (select catalog_xmin::text
                      from pg_replication_slots
                     where catalog_xmin is not null
                       and age(catalog_xmin)::bigint >= $1
                     order by age(catalog_xmin) desc
                     limit 1) as oldest_catalog_xmin,
                   (select age(catalog_xmin)::bigint
                      from pg_replication_slots
                     where catalog_xmin is not null
                       and age(catalog_xmin)::bigint >= $1
                     order by age(catalog_xmin) desc
                     limit 1) as oldest_catalog_xmin_age,
                   (select count(*)::bigint
                      from pg_replication_slots
                     where catalog_xmin is not null
                       and age(catalog_xmin)::bigint >= $1) as stale_catalog_xmin_slot_count,
                   (select count(*)::bigint
                      from pg_stat_activity
                     where xact_start is not null
                       and state <> 'idle'
                       and now() - xact_start >= interval '5 minutes') as long_running_transaction_count,
                   (select floor(extract(epoch from max(now() - xact_start)))::bigint
                      from pg_stat_activity
                     where xact_start is not null
                       and state <> 'idle'
                       and now() - xact_start >= interval '5 minutes') as oldest_transaction_age_seconds,
                   (select count(*)::bigint
                      from pg_prepared_xacts) as prepared_transaction_count,
                   (select floor(extract(epoch from max(now() - prepared)))::bigint
                      from pg_prepared_xacts) as oldest_prepared_transaction_age_seconds,
                   (select count(*)::bigint
                      from pg_stat_replication
                     where backend_xmin is not null) as hot_standby_feedback_replica_count,
                   (select max(age(backend_xmin))::bigint
                      from pg_stat_replication
                     where backend_xmin is not null) as oldest_hot_standby_feedback_xmin_age
            "#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wal_projection_samples_current_lsn_and_diff() {
        assert!(CURRENT_WAL_LSN_SQL.contains("pg_current_wal_lsn()"));
        assert!(WAL_LSN_DIFF_SQL.contains("pg_wal_lsn_diff"));
        assert_eq!(WAL_RATE_SAMPLE_MS, 2_000);
    }

    #[test]
    fn wraparound_probe_reads_oldest_database_age() {
        assert!(TXID_WRAPAROUND_SQL.contains("age(datfrozenxid)"));
        assert!(TXID_WRAPAROUND_SQL.contains("autovacuum_freeze_max_age"));
        assert!(TXID_WRAPAROUND_SQL.contains("pg_database"));
    }

    #[test]
    fn xmin_probe_covers_known_vacuum_pinners() {
        assert!(XMIN_HORIZON_SQL.contains("catalog_xmin"));
        assert!(XMIN_HORIZON_SQL.contains("stale_catalog_xmin_slot_count"));
        assert!(XMIN_HORIZON_SQL.contains(">= $1"));
        assert_eq!(STALE_CATALOG_XMIN_AGE_TXIDS, 1_000_000);
        assert!(XMIN_HORIZON_SQL.contains("pg_stat_activity"));
        assert!(XMIN_HORIZON_SQL.contains("pg_prepared_xacts"));
        assert!(XMIN_HORIZON_SQL.contains("backend_xmin"));
        assert!(SLOT_CATALOG_XMIN_SQL.contains("pg_replication_slots"));
    }
}
