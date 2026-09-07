use trellara_pg_capture::{
    ReplicationSlotStatus, TransactionIdWraparoundStatus, XminHorizonStatus,
};

use crate::{
    display_optional_duration, display_optional_i64, format_duration,
    source_safety::types::SourceSafetyFactor, source_slot_position_evidence, FlowAlertSeverity,
};

const WAL_HEADROOM_WARN_SECONDS: i64 = 24 * 60 * 60;
const TXID_WRAPAROUND_WARNING_PERCENT: u8 = 75;
const TXID_WRAPAROUND_CRITICAL_PERCENT: u8 = 90;

pub(crate) fn source_wal_retention_factor(
    source_slot: &ReplicationSlotStatus,
    wal_retention_warn_bytes: Option<i64>,
    recommendation: &'static str,
) -> Option<SourceSafetyFactor> {
    if let Some(projection) = &source_slot.wal_headroom {
        let headroom_seconds = projection.headroom_seconds?;
        if headroom_seconds <= WAL_HEADROOM_WARN_SECONDS {
            return Some(SourceSafetyFactor::warning(
                "source_wal_retention_risk",
                20,
                format!(
                    "source slot {} has {} of safe WAL headroom at the current write rate; {}",
                    source_slot.slot_name,
                    format_duration(headroom_seconds),
                    source_slot_position_evidence(source_slot)
                ),
                recommendation,
            ));
        }
        return None;
    }

    wal_retention_warn_bytes
        .zip(source_slot.retained_wal_bytes)
        .filter(|(threshold, retained)| retained >= threshold)
        .map(|(threshold, retained)| {
            SourceSafetyFactor::warning(
                "source_wal_retention_risk",
                20,
                format!(
                    "source slot {} retained WAL {} bytes is at or above warning threshold {} bytes; {}",
                    source_slot.slot_name,
                    retained,
                    threshold,
                    source_slot_position_evidence(source_slot)
                ),
                recommendation,
            )
        })
}

pub(crate) fn source_postgres_risk_factors(
    source_slot: &ReplicationSlotStatus,
) -> Vec<SourceSafetyFactor> {
    let mut factors = Vec::new();
    if let Some(wraparound) = &source_slot.transaction_id_wraparound {
        if let Some(factor) = transaction_id_wraparound_factor(wraparound) {
            factors.push(factor);
        }
    }
    if let Some(xmin_horizon) = &source_slot.xmin_horizon {
        if let Some(factor) = xmin_horizon_factor(source_slot, xmin_horizon) {
            factors.push(factor);
        }
    }
    factors
}

pub(crate) fn source_slot_projected_wal_pressure_active(
    source_slot: &ReplicationSlotStatus,
) -> bool {
    source_slot
        .wal_headroom
        .as_ref()
        .and_then(|projection| projection.headroom_seconds)
        .is_some_and(|seconds| seconds <= WAL_HEADROOM_WARN_SECONDS)
}

fn transaction_id_wraparound_factor(
    wraparound: &TransactionIdWraparoundStatus,
) -> Option<SourceSafetyFactor> {
    let usage_percent = wraparound.usage_percent?;
    if usage_percent < TXID_WRAPAROUND_WARNING_PERCENT {
        return None;
    }

    let evidence = format!(
        "transaction ID wraparound headroom is {}% used on database {} (oldest_datfrozenxid_age={} autovacuum_freeze_max_age={} remaining_transactions={})",
        usage_percent,
        wraparound.oldest_database.as_deref().unwrap_or("unknown"),
        display_optional_i64(wraparound.oldest_datfrozenxid_age),
        display_optional_i64(wraparound.autovacuum_freeze_max_age),
        display_optional_i64(wraparound.remaining_transactions)
    );
    let recommendation =
        "unblock vacuum by draining stale slots and long transactions before transaction ID wraparound can force source writes offline";

    if usage_percent >= TXID_WRAPAROUND_CRITICAL_PERCENT {
        Some(SourceSafetyFactor::critical(
            "source_transaction_id_wraparound_risk",
            35,
            evidence,
            recommendation,
        ))
    } else {
        Some(SourceSafetyFactor::warning(
            "source_transaction_id_wraparound_risk",
            20,
            evidence,
            recommendation,
        ))
    }
}

fn xmin_horizon_factor(
    source_slot: &ReplicationSlotStatus,
    xmin_horizon: &XminHorizonStatus,
) -> Option<SourceSafetyFactor> {
    if xmin_horizon.stale_catalog_xmin_slot_count == 0
        && xmin_horizon.long_running_transaction_count == 0
        && xmin_horizon.prepared_transaction_count == 0
        && xmin_horizon.hot_standby_feedback_replica_count == 0
    {
        return None;
    }

    Some(SourceSafetyFactor::warning(
            "source_xmin_horizon_risk",
            20,
            format!(
            "source slot {} has vacuum horizon pinners: slot_catalog_xmin={} stale_catalog_xmin_slots={} oldest_catalog_xmin_slot={} oldest_catalog_xmin_age={} long_running_transactions={} oldest_transaction={} prepared_transactions={} oldest_prepared={} hot_standby_feedback_replicas={} oldest_hot_standby_feedback_xmin_age={}",
            source_slot.slot_name,
            xmin_horizon.slot_catalog_xmin.as_deref().unwrap_or("none"),
            xmin_horizon.stale_catalog_xmin_slot_count,
            xmin_horizon
                .oldest_catalog_xmin_slot
                .as_deref()
                .unwrap_or("none"),
            display_optional_i64(xmin_horizon.oldest_catalog_xmin_age),
            xmin_horizon.long_running_transaction_count,
            display_optional_duration(xmin_horizon.oldest_transaction_age_seconds),
            xmin_horizon.prepared_transaction_count,
            display_optional_duration(xmin_horizon.oldest_prepared_transaction_age_seconds),
            xmin_horizon.hot_standby_feedback_replica_count,
            display_optional_i64(xmin_horizon.oldest_hot_standby_feedback_xmin_age)
        ),
        "finish long-running transactions, resolve prepared transactions, retire stale slots, or review hot_standby_feedback before vacuum bloat becomes a source outage",
    ))
}

pub(crate) fn strongest_source_factor(
    factors: Vec<SourceSafetyFactor>,
) -> Option<SourceSafetyFactor> {
    let mut first = None;
    for factor in factors {
        if factor.severity == FlowAlertSeverity::Critical {
            return Some(factor);
        }
        if first.is_none() {
            first = Some(factor);
        }
    }
    first
}
