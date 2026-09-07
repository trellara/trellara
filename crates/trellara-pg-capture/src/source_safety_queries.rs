use serde::Serialize;

use crate::{
    source_discovery::{DISCOVER_USER_TABLES_SQL, INSPECT_LOGICAL_REPLICATION_SLOTS_SQL},
    source_inspection::{INSPECT_SLOT_STATUS_SQL, INSPECT_TABLES_SQL},
    source_postgres_risk_sql::{
        CURRENT_WAL_LSN_SQL, SLOT_CATALOG_XMIN_SQL, TXID_WRAPAROUND_SQL, WAL_LSN_DIFF_SQL,
        XMIN_HORIZON_SQL,
    },
    source_subscription_inspection::{
        SUBSCRIPTION_CONFLICT_STATS_SQL, SUBSCRIPTION_STATS_VIEW_SQL,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceSafetyReadOnlyQuery {
    pub name: &'static str,
    pub when: &'static str,
    pub parameters: &'static str,
    pub sql: &'static str,
}

pub fn source_safety_read_only_queries() -> &'static [SourceSafetyReadOnlyQuery] {
    READ_ONLY_QUERIES
}

const WAL_SAMPLE_WHEN: &str =
    "slot exists and safe_wal_size is positive; sampled twice over 2000ms";
const SLOT_PARAMETER: &str = "$1 = logical replication slot name";
const TABLE_PARAMETERS: &str = "$1 = selected schema names, $2 = selected table names";
const XMIN_PARAMETERS: &str = "$1 = stale catalog_xmin cutoff in transactions (currently 1000000)";

const READ_ONLY_QUERIES: &[SourceSafetyReadOnlyQuery] = &[
    SourceSafetyReadOnlyQuery {
        name: "discover_user_tables",
        when: "trellara-check runs without a config or selected tables",
        parameters: "none",
        sql: DISCOVER_USER_TABLES_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "table_replica_identity_preflight",
        when: "direct database source-safety checks selected tables",
        parameters: TABLE_PARAMETERS,
        sql: INSPECT_TABLES_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "replication_slot_status",
        when: "source-safety checks the configured slot",
        parameters: SLOT_PARAMETER,
        sql: INSPECT_SLOT_STATUS_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "logical_replication_slots",
        when: "trellara-check runs without a configured slot name",
        parameters: "none",
        sql: INSPECT_LOGICAL_REPLICATION_SLOTS_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "wal_headroom_current_lsn_sample",
        when: WAL_SAMPLE_WHEN,
        parameters: "none; Trellara waits 2000ms before the second sample",
        sql: CURRENT_WAL_LSN_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "wal_headroom_bytes_between_samples",
        when: "after two pg_current_wal_lsn() samples are collected",
        parameters: "$1 = later LSN, $2 = earlier LSN",
        sql: WAL_LSN_DIFF_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "transaction_id_wraparound_budget",
        when: "slot exists; best-effort incident-prevention probe",
        parameters: "none",
        sql: TXID_WRAPAROUND_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "slot_catalog_xmin",
        when: "slot exists; best-effort vacuum horizon probe",
        parameters: SLOT_PARAMETER,
        sql: SLOT_CATALOG_XMIN_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "xmin_horizon_pinners",
        when: "slot exists; best-effort vacuum bloat and wraparound pinner probe",
        parameters: XMIN_PARAMETERS,
        sql: XMIN_HORIZON_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "subscription_stats_view_probe",
        when: "source-safety checks whether PostgreSQL exposes subscription conflict stats",
        parameters: "none",
        sql: SUBSCRIPTION_STATS_VIEW_SQL,
    },
    SourceSafetyReadOnlyQuery {
        name: "subscription_conflict_counters",
        when: "pg_catalog.pg_stat_subscription_stats exists",
        parameters: "none",
        sql: SUBSCRIPTION_CONFLICT_STATS_SQL,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_safety_query_manifest_covers_postgres_risk_story() {
        let names = READ_ONLY_QUERIES
            .iter()
            .map(|query| query.name)
            .collect::<Vec<_>>();

        assert!(names.contains(&"wal_headroom_current_lsn_sample"));
        assert!(names.contains(&"logical_replication_slots"));
        assert!(names.contains(&"transaction_id_wraparound_budget"));
        assert!(names.contains(&"xmin_horizon_pinners"));
        assert_eq!(crate::source_postgres_risk_sql::WAL_RATE_SAMPLE_MS, 2_000);
        assert_eq!(
            crate::source_postgres_risk_sql::STALE_CATALOG_XMIN_AGE_TXIDS,
            1_000_000
        );
    }

    #[test]
    fn source_safety_query_manifest_is_read_only() {
        for query in READ_ONLY_QUERIES {
            let sql = query.sql.to_ascii_lowercase();
            assert!(sql.contains("select"));
            assert!(!sql.contains(" insert "));
            assert!(!sql.contains(" update "));
            assert!(!sql.contains(" delete "));
            assert!(!sql.contains(" alter "));
            assert!(!sql.contains(" drop "));
            assert!(!sql.contains(" create "));
        }
    }
}
