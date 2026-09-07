use std::time::Duration;

use tokio_postgres::Client;

use crate::source_postgres_risk_sql::{
    CURRENT_WAL_LSN_SQL, SLOT_CATALOG_XMIN_SQL, STALE_CATALOG_XMIN_AGE_TXIDS, TXID_WRAPAROUND_SQL,
    WAL_LSN_DIFF_SQL, WAL_RATE_SAMPLE_MS, XMIN_HORIZON_SQL,
};
use crate::{
    ReplicationSlotStatus, Result, TransactionIdWraparoundStatus, WalHeadroomProjection,
    XminHorizonStatus,
};

pub(crate) async fn attach_postgres_risk_evidence(
    client: &Client,
    status: &mut ReplicationSlotStatus,
) {
    attach_postgres_risk_evidence_to_slots(client, std::slice::from_mut(status)).await;
}

pub(crate) async fn attach_postgres_risk_evidence_to_slots(
    client: &Client,
    statuses: &mut [ReplicationSlotStatus],
) {
    if statuses.iter().all(|status| !status.exists) {
        return;
    }

    let wal_rate = if statuses
        .iter()
        .any(|status| status.safe_wal_size_bytes.is_some_and(|bytes| bytes > 0))
    {
        inspect_wal_bytes_per_second(client).await.ok()
    } else {
        None
    };
    let wraparound = inspect_transaction_id_wraparound(client).await.ok();
    let xmin_horizon = inspect_xmin_horizon_pinners(client).await.ok();

    for status in statuses {
        if !status.exists {
            continue;
        }
        status.wal_headroom =
            wal_rate.and_then(|rate| wal_headroom_projection(status.safe_wal_size_bytes, rate));
        status.transaction_id_wraparound = wraparound.clone();
        status.xmin_horizon = inspect_slot_xmin_horizon(client, &status.slot_name, &xmin_horizon)
            .await
            .ok();
    }
}

async fn inspect_wal_bytes_per_second(client: &Client) -> Result<i64> {
    let start_lsn = current_wal_lsn(client).await?;
    tokio::time::sleep(Duration::from_millis(WAL_RATE_SAMPLE_MS)).await;
    let end_lsn = current_wal_lsn(client).await?;
    let row = client
        .query_one(WAL_LSN_DIFF_SQL, &[&end_lsn, &start_lsn])
        .await?;
    let sampled_wal_bytes = row.get::<_, i64>("wal_bytes").max(0);
    Ok((sampled_wal_bytes * 1_000 / i64::try_from(WAL_RATE_SAMPLE_MS).unwrap_or(1)).max(0))
}

fn wal_headroom_projection(
    safe_wal_size_bytes: Option<i64>,
    wal_bytes_per_second: i64,
) -> Option<WalHeadroomProjection> {
    let safe_wal_size_bytes = safe_wal_size_bytes.filter(|bytes| *bytes > 0)?;
    let headroom_seconds = if wal_bytes_per_second > 0 {
        Some(safe_wal_size_bytes / wal_bytes_per_second)
    } else {
        None
    };

    Some(WalHeadroomProjection {
        source: "pg_current_wal_lsn_sample".to_string(),
        safe_wal_size_bytes,
        wal_bytes_per_second,
        sample_ms: i64::try_from(WAL_RATE_SAMPLE_MS).unwrap_or(0),
        headroom_seconds,
    })
}

async fn current_wal_lsn(client: &Client) -> Result<String> {
    let row = client.query_one(CURRENT_WAL_LSN_SQL, &[]).await?;
    Ok(row.get("current_lsn"))
}

async fn inspect_transaction_id_wraparound(
    client: &Client,
) -> Result<TransactionIdWraparoundStatus> {
    let row = client.query_one(TXID_WRAPAROUND_SQL, &[]).await?;
    let oldest_datfrozenxid_age = row.get::<_, Option<i64>>("oldest_datfrozenxid_age");
    let autovacuum_freeze_max_age = row.get::<_, Option<i64>>("autovacuum_freeze_max_age");
    let remaining_transactions = oldest_datfrozenxid_age.zip(autovacuum_freeze_max_age).map(
        |(oldest_datfrozenxid_age, autovacuum_freeze_max_age)| {
            autovacuum_freeze_max_age - oldest_datfrozenxid_age
        },
    );
    let usage_percent = oldest_datfrozenxid_age
        .zip(autovacuum_freeze_max_age)
        .filter(|(_, autovacuum_freeze_max_age)| *autovacuum_freeze_max_age > 0)
        .map(|(oldest_datfrozenxid_age, autovacuum_freeze_max_age)| {
            ((oldest_datfrozenxid_age * 100) / autovacuum_freeze_max_age).clamp(0, 100) as u8
        });

    Ok(TransactionIdWraparoundStatus {
        oldest_database: row.get("oldest_database"),
        oldest_datfrozenxid_age,
        autovacuum_freeze_max_age,
        remaining_transactions,
        usage_percent,
    })
}

async fn inspect_xmin_horizon_pinners(client: &Client) -> Result<XminHorizonStatus> {
    let row = client
        .query_one(XMIN_HORIZON_SQL, &[&STALE_CATALOG_XMIN_AGE_TXIDS])
        .await?;

    Ok(XminHorizonStatus {
        slot_catalog_xmin: None,
        oldest_catalog_xmin_slot: row.get("oldest_catalog_xmin_slot"),
        oldest_catalog_xmin: row.get("oldest_catalog_xmin"),
        oldest_catalog_xmin_age: row.get("oldest_catalog_xmin_age"),
        stale_catalog_xmin_slot_count: row.get("stale_catalog_xmin_slot_count"),
        long_running_transaction_count: row.get("long_running_transaction_count"),
        oldest_transaction_age_seconds: row.get("oldest_transaction_age_seconds"),
        prepared_transaction_count: row.get("prepared_transaction_count"),
        oldest_prepared_transaction_age_seconds: row.get("oldest_prepared_transaction_age_seconds"),
        hot_standby_feedback_replica_count: row.get("hot_standby_feedback_replica_count"),
        oldest_hot_standby_feedback_xmin_age: row.get("oldest_hot_standby_feedback_xmin_age"),
    })
}

async fn inspect_slot_xmin_horizon(
    client: &Client,
    slot_name: &str,
    horizon: &Option<XminHorizonStatus>,
) -> Result<XminHorizonStatus> {
    let Some(mut horizon) = horizon.clone() else {
        return inspect_xmin_horizon_pinners(client).await;
    };
    let slot_row = client
        .query_opt(SLOT_CATALOG_XMIN_SQL, &[&slot_name])
        .await?;
    horizon.slot_catalog_xmin = slot_row.and_then(|row| row.get("slot_catalog_xmin"));
    Ok(horizon)
}
