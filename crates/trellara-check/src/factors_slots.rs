use trellara_pg_capture::ReplicationSlotStatus;

use crate::{
    factors_slot_failover::slot_failover_factors,
    factors_slot_recommendation::source_slot_issue_recommendation,
    labels::format_duration,
    slot_evidence::{slot_position_evidence, wal_headroom_label},
    CheckFactor,
};

const WAL_HEADROOM_WARN_SECONDS: i64 = 24 * 60 * 60;

pub(crate) fn slot_factors(slot: &ReplicationSlotStatus) -> Vec<CheckFactor> {
    let mut factors = Vec::new();

    if !slot.exists {
        factors.push(CheckFactor::critical(
            "source_slot_missing",
            35,
            format!(
                "source replication slot {} does not exist; {}",
                slot.slot_name,
                slot_position_evidence(slot)
            ),
            "create or select a logical replication slot before starting CDC",
        ));
        return factors;
    }

    if slot.plugin.as_deref() != Some(slot.expected_plugin.as_str()) {
        factors.push(CheckFactor::critical(
            "source_slot_plugin_mismatch",
            35,
            format!(
                "source replication slot {} uses plugin {}, expected {}; {}",
                slot.slot_name,
                slot.plugin.as_deref().unwrap_or("unknown"),
                slot.expected_plugin,
                slot_position_evidence(slot)
            ),
            "create or select a replication slot with the expected logical decoding plugin before starting CDC",
        ));
    }

    push_wal_boundary_factors(slot, &mut factors);
    push_wal_headroom_factor(slot, &mut factors);
    factors.extend(slot_failover_factors(slot));

    if factors.is_empty() && !slot.issues.is_empty() {
        factors.push(CheckFactor::critical(
            "source_slot_unsafe",
            35,
            format!(
                "source replication slot {} has {} issue(s): {}; {}",
                slot.slot_name,
                slot.issues.len(),
                slot.issues.join("; "),
                slot_position_evidence(slot)
            ),
            source_slot_issue_recommendation(slot),
        ));
    }

    if factors.is_empty() && slot.active == Some(false) {
        factors.push(CheckFactor::warning(
            "source_slot_inactive",
            10,
            format!(
                "source replication slot {} is inactive and may be pinning WAL; {}",
                slot.slot_name,
                slot_position_evidence(slot)
            ),
            "confirm the slot is intentionally idle, drop abandoned slots, or restart the consumer before WAL pressure builds",
        ));
    }

    factors
}

fn push_wal_boundary_factors(slot: &ReplicationSlotStatus, factors: &mut Vec<CheckFactor>) {
    if let Some(reason) = slot
        .invalidation_reason
        .as_deref()
        .filter(|reason| !reason.trim().is_empty())
    {
        factors.push(CheckFactor::critical(
            "source_slot_invalidated",
            35,
            format!(
                "source replication slot {} was invalidated: {}; {}",
                slot.slot_name,
                reason,
                slot_position_evidence(slot)
            ),
            "recreate the source replication slot, reseed affected targets, then resume CDC from the fresh handoff",
        ));
    } else if slot.wal_status.as_deref() == Some("lost") {
        factors.push(CheckFactor::critical(
            "source_slot_wal_lost",
            35,
            format!(
                "source replication slot {} has wal_status=lost; {}",
                slot.slot_name,
                slot_position_evidence(slot)
            ),
            "recreate the source replication slot, reseed affected targets, then resume CDC from the fresh handoff",
        ));
    } else {
        push_wal_pressure_factors(slot, factors);
    }

    if slot.retained_wal_bytes.is_some_and(|bytes| bytes < 0) {
        factors.push(CheckFactor::warning(
            "source_slot_restart_lsn_ahead",
            10,
            format!(
                "source replication slot {} has restart_lsn ahead of current WAL; {}",
                slot.slot_name,
                slot_position_evidence(slot)
            ),
            "inspect the source replication slot and WAL position before trusting CDC progress",
        ));
    }
}

fn push_wal_pressure_factors(slot: &ReplicationSlotStatus, factors: &mut Vec<CheckFactor>) {
    if slot.wal_status.as_deref() == Some("unreserved") {
        factors.push(CheckFactor::warning(
            "source_slot_wal_unreserved",
            20,
            format!(
                "source replication slot {} has wal_status=unreserved and may lose required WAL; {}",
                slot.slot_name,
                slot_position_evidence(slot)
            ),
            "pause unsafe consumers, drain relay/apply lag, or reseed affected targets before the source slot loses WAL",
        ));
    }
    if slot.safe_wal_size_bytes.is_some_and(|bytes| bytes <= 0) {
        factors.push(CheckFactor::warning(
            "source_slot_safe_wal_exhausted",
            20,
            format!(
                "source replication slot {} has exhausted safe_wal_size; {}",
                slot.slot_name,
                slot_position_evidence(slot)
            ),
            "pause unsafe consumers, drain relay/apply lag, or reseed affected targets before the source slot loses WAL",
        ));
    }
}

fn push_wal_headroom_factor(slot: &ReplicationSlotStatus, factors: &mut Vec<CheckFactor>) {
    let Some(projection) = &slot.wal_headroom else {
        return;
    };
    let Some(headroom_seconds) = projection.headroom_seconds else {
        return;
    };
    if headroom_seconds > WAL_HEADROOM_WARN_SECONDS {
        return;
    }
    factors.push(CheckFactor::warning(
        "source_wal_retention_risk",
        20,
        format!(
            "source slot {} has {} of safe WAL headroom at the current write rate; {}",
            slot.slot_name,
            format_duration(headroom_seconds),
            wal_headroom_label(slot).unwrap_or_else(|| slot_position_evidence(slot))
        ),
        "move CDC pressure off the primary, drain stale slots, or increase WAL retention before headroom runs out",
    ));
}
