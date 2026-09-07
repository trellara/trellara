use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReplicationSlotStatus {
    pub slot_name: String,
    pub exists: bool,
    pub plugin: Option<String>,
    pub expected_plugin: String,
    pub active: Option<bool>,
    pub failover: Option<bool>,
    pub synced: Option<bool>,
    pub inactive_since: Option<String>,
    pub idle_replication_slot_timeout: Option<String>,
    pub restart_lsn: Option<String>,
    pub confirmed_flush_lsn: Option<String>,
    pub retained_wal_bytes: Option<i64>,
    pub wal_status: Option<String>,
    pub safe_wal_size_bytes: Option<i64>,
    pub invalidation_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wal_headroom: Option<WalHeadroomProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_id_wraparound: Option<TransactionIdWraparoundStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xmin_horizon: Option<XminHorizonStatus>,
    pub issues: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct WalHeadroomProjection {
    pub source: String,
    pub safe_wal_size_bytes: i64,
    pub wal_bytes_per_second: i64,
    pub sample_ms: i64,
    pub headroom_seconds: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TransactionIdWraparoundStatus {
    pub oldest_database: Option<String>,
    pub oldest_datfrozenxid_age: Option<i64>,
    pub autovacuum_freeze_max_age: Option<i64>,
    pub remaining_transactions: Option<i64>,
    pub usage_percent: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct XminHorizonStatus {
    pub slot_catalog_xmin: Option<String>,
    pub oldest_catalog_xmin_slot: Option<String>,
    pub oldest_catalog_xmin: Option<String>,
    pub oldest_catalog_xmin_age: Option<i64>,
    pub stale_catalog_xmin_slot_count: i64,
    pub long_running_transaction_count: i64,
    pub oldest_transaction_age_seconds: Option<i64>,
    pub prepared_transaction_count: i64,
    pub oldest_prepared_transaction_age_seconds: Option<i64>,
    pub hot_standby_feedback_replica_count: i64,
    pub oldest_hot_standby_feedback_xmin_age: Option<i64>,
}
