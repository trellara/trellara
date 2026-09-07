pub(crate) mod evidence;
pub(crate) mod failover;
pub(crate) mod inactive;
pub(crate) mod text;

use trellara_pg_capture::ReplicationSlotStatus;

use crate::{
    source_safety::types::SourceSafetyFactor, source_slot_position_evidence,
    source_slot_projected_wal_pressure_active,
};

pub(crate) fn source_slot_issue_recommendation(
    source_slot: &ReplicationSlotStatus,
) -> &'static str {
    if source_slot_wal_boundary_broken(source_slot) {
        "recreate the source replication slot, run trellara reseed for affected targets, then resume CDC from the fresh handoff"
    } else if source_slot
        .issues
        .iter()
        .any(|issue| issue.contains("unreserved") || issue.contains("safe WAL size is exhausted"))
    {
        "pause unsafe consumers, drain relay/apply lag, or reseed affected targets before the source slot loses WAL"
    } else {
        "inspect trellara preflight and repair the source replication configuration"
    }
}

pub(crate) fn source_slot_issue_factors(
    source_slot: &ReplicationSlotStatus,
) -> Vec<SourceSafetyFactor> {
    let mut factors = Vec::new();

    if source_slot.plugin.as_deref() != Some(source_slot.expected_plugin.as_str()) {
        factors.push(SourceSafetyFactor::critical(
            "source_slot_plugin_mismatch",
            35,
            format!(
                "source replication slot {} uses plugin {}, expected {}; {}",
                source_slot.slot_name,
                source_slot.plugin.as_deref().unwrap_or("unknown"),
                source_slot.expected_plugin,
                source_slot_position_evidence(source_slot)
            ),
            "create or select a replication slot with the expected logical decoding plugin before starting CDC",
        ));
    }

    if let Some(reason) = source_slot
        .invalidation_reason
        .as_deref()
        .filter(|reason| !reason.trim().is_empty())
    {
        factors.push(SourceSafetyFactor::critical(
            "source_slot_invalidated",
            35,
            format!(
                "source replication slot {} was invalidated: {}; {}",
                source_slot.slot_name,
                reason,
                source_slot_position_evidence(source_slot)
            ),
            "recreate the source replication slot, run trellara reseed for affected targets, then resume CDC from the fresh handoff",
        ));
    } else if source_slot.wal_status.as_deref() == Some("lost") {
        factors.push(SourceSafetyFactor::critical(
            "source_slot_wal_lost",
            35,
            format!(
                "source replication slot {} has wal_status=lost; {}",
                source_slot.slot_name,
                source_slot_position_evidence(source_slot)
            ),
            "recreate the source replication slot, run trellara reseed for affected targets, then resume CDC from the fresh handoff",
        ));
    } else {
        if source_slot.wal_status.as_deref() == Some("unreserved") {
            factors.push(SourceSafetyFactor::warning(
                "source_slot_wal_unreserved",
                20,
                format!(
                    "source replication slot {} has wal_status=unreserved and may lose required WAL; {}",
                    source_slot.slot_name,
                    source_slot_position_evidence(source_slot)
                ),
                "pause unsafe consumers, drain relay/apply lag, or reseed affected targets before the source slot loses WAL",
            ));
        }

        if source_slot
            .safe_wal_size_bytes
            .is_some_and(|bytes| bytes <= 0)
        {
            factors.push(SourceSafetyFactor::warning(
                "source_slot_safe_wal_exhausted",
                20,
                format!(
                    "source replication slot {} has exhausted safe_wal_size; {}",
                    source_slot.slot_name,
                    source_slot_position_evidence(source_slot)
                ),
                "pause unsafe consumers, drain relay/apply lag, or reseed affected targets before the source slot loses WAL",
            ));
        }
    }

    if source_slot
        .retained_wal_bytes
        .is_some_and(|bytes| bytes < 0)
    {
        factors.push(SourceSafetyFactor::warning(
            "source_slot_restart_lsn_ahead",
            10,
            format!(
                "source replication slot {} has restart_lsn ahead of current WAL; {}",
                source_slot.slot_name,
                source_slot_position_evidence(source_slot)
            ),
            "inspect the source replication slot and WAL position before trusting CDC progress",
        ));
    }

    if factors.is_empty() && !source_slot.issues.is_empty() {
        factors.push(SourceSafetyFactor::critical(
            "source_slot_unsafe",
            35,
            format!(
                "source replication slot {} has {} issue(s): {}; {}",
                source_slot.slot_name,
                source_slot.issues.len(),
                source_slot.issues.join("; "),
                source_slot_position_evidence(source_slot)
            ),
            source_slot_issue_recommendation(source_slot),
        ));
    }

    factors
}

pub(crate) fn source_slot_wal_boundary_broken(source_slot: &ReplicationSlotStatus) -> bool {
    source_slot
        .issues
        .iter()
        .any(|issue| issue.contains("WAL is lost") || issue.contains("invalidated"))
}

pub(crate) fn source_slot_wal_pressure_active(source_slot: &ReplicationSlotStatus) -> bool {
    source_slot.wal_status.as_deref() == Some("unreserved")
        || source_slot
            .safe_wal_size_bytes
            .is_some_and(|bytes| bytes <= 0)
        || source_slot_projected_wal_pressure_active(source_slot)
}
