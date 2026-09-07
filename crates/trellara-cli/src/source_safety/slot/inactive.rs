use trellara_pg_capture::ReplicationSlotStatus;

use crate::source_slot_position_evidence;

pub(crate) fn source_slot_inactive_evidence(source_slot: &ReplicationSlotStatus) -> String {
    let timeout = source_slot_idle_timeout_display(source_slot);
    match source_slot.inactive_since.as_deref() {
        Some(inactive_since) if !inactive_since.trim().is_empty() => {
            if let Some(timeout) = timeout {
                format!(
                    "source replication slot {} exists but has been inactive since {}; idle_replication_slot_timeout is {}; {}",
                    source_slot.slot_name,
                    inactive_since,
                    timeout,
                    source_slot_position_evidence(source_slot)
                )
            } else {
                format!(
                    "source replication slot {} exists but has been inactive since {}; {}",
                    source_slot.slot_name,
                    inactive_since,
                    source_slot_position_evidence(source_slot)
                )
            }
        }
        _ => format!(
            "source replication slot {} exists but is inactive; {}",
            source_slot.slot_name,
            source_slot_position_evidence(source_slot)
        ),
    }
}

pub(crate) fn source_slot_inactive_recommendation(
    source_slot: &ReplicationSlotStatus,
) -> &'static str {
    if source_slot_idle_timeout_enabled(source_slot) {
        "confirm the slot is intentionally idle; Postgres idle slot cleanup is configured, but operators should still remove abandoned slots deliberately"
    } else {
        "confirm the slot is intentionally idle; configure idle_replication_slot_timeout or drop abandoned slots so they cannot retain WAL indefinitely"
    }
}

fn source_slot_idle_timeout_enabled(source_slot: &ReplicationSlotStatus) -> bool {
    source_slot
        .idle_replication_slot_timeout
        .as_deref()
        .is_some_and(|timeout| !slot_timeout_is_disabled(timeout))
}

fn source_slot_idle_timeout_display(source_slot: &ReplicationSlotStatus) -> Option<&str> {
    source_slot
        .idle_replication_slot_timeout
        .as_deref()
        .filter(|timeout| !timeout.trim().is_empty())
}

fn slot_timeout_is_disabled(timeout: &str) -> bool {
    let normalized = timeout.trim().to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "0" | "0s" | "0ms" | "0 sec" | "0 secs" | "0 second" | "0 seconds"
    )
}
