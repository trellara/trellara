use trellara_pg_capture::{
    ReplicationSlotStatus, TransactionIdWraparoundStatus, XminHorizonStatus,
};

use crate::{
    labels::{display_optional_duration, display_optional_i64},
    CheckFactor,
};

const TXID_WRAPAROUND_WARNING_PERCENT: u8 = 75;
const TXID_WRAPAROUND_CRITICAL_PERCENT: u8 = 90;

pub(crate) fn postgres_risk_factors(slots: &[ReplicationSlotStatus]) -> Vec<CheckFactor> {
    let mut factors = Vec::new();
    if let Some(factor) = slots
        .iter()
        .filter_map(|slot| slot.transaction_id_wraparound.as_ref())
        .find_map(transaction_id_wraparound_factor)
    {
        factors.push(factor);
    }
    if let Some((slot, xmin_horizon)) = slots
        .iter()
        .find_map(|slot| slot.xmin_horizon.as_ref().map(|xmin| (slot, xmin)))
    {
        if let Some(factor) = xmin_horizon_factor(slot, xmin_horizon) {
            factors.push(factor);
        }
    }
    factors
}

fn transaction_id_wraparound_factor(
    wraparound: &TransactionIdWraparoundStatus,
) -> Option<CheckFactor> {
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
        Some(CheckFactor::critical(
            "source_transaction_id_wraparound_risk",
            35,
            evidence,
            recommendation,
        ))
    } else {
        Some(CheckFactor::warning(
            "source_transaction_id_wraparound_risk",
            20,
            evidence,
            recommendation,
        ))
    }
}

fn xmin_horizon_factor(
    slot: &ReplicationSlotStatus,
    xmin_horizon: &XminHorizonStatus,
) -> Option<CheckFactor> {
    if xmin_horizon.stale_catalog_xmin_slot_count == 0
        && xmin_horizon.long_running_transaction_count == 0
        && xmin_horizon.prepared_transaction_count == 0
        && xmin_horizon.hot_standby_feedback_replica_count == 0
    {
        return None;
    }

    Some(CheckFactor::warning(
        "source_xmin_horizon_risk",
        20,
        format!(
            "source slot {} has vacuum horizon pinners: slot_catalog_xmin={} stale_catalog_xmin_slots={} oldest_catalog_xmin_slot={} oldest_catalog_xmin_age={} long_running_transactions={} oldest_transaction={} prepared_transactions={} oldest_prepared={} hot_standby_feedback_replicas={} oldest_hot_standby_feedback_xmin_age={}",
            slot.slot_name,
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
