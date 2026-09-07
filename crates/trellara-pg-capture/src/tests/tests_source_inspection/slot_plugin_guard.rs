use crate::source_slot_inspection::fail_on_slot_plugin_mismatch;

use super::*;

#[test]
fn slot_plugin_guard_rejects_existing_mismatched_slot_before_capture_start() {
    let status = replication_slot_status(true, Some("test_decoding"));

    assert!(matches!(
        fail_on_slot_plugin_mismatch(&status),
        Err(CaptureError::SlotPluginMismatch {
            slot_name,
            expected_plugin,
            actual_plugin
        }) if slot_name == "trellara_slot"
            && expected_plugin == "pgoutput"
            && actual_plugin == "test_decoding"
    ));
}

#[test]
fn slot_plugin_guard_rejects_existing_slot_with_unknown_plugin() {
    let status = replication_slot_status(true, None);

    assert!(matches!(
        fail_on_slot_plugin_mismatch(&status),
        Err(CaptureError::SlotPluginMismatch {
            slot_name,
            expected_plugin,
            actual_plugin
        }) if slot_name == "trellara_slot"
            && expected_plugin == "pgoutput"
            && actual_plugin == "unknown"
    ));
}

#[test]
fn slot_plugin_guard_allows_matching_or_missing_slots() {
    for (exists, plugin) in [(true, Some("pgoutput".to_string())), (false, None)] {
        let status = replication_slot_status(exists, plugin.as_deref());

        fail_on_slot_plugin_mismatch(&status).expect("slot is acceptable");
    }
}

fn replication_slot_status(exists: bool, plugin: Option<&str>) -> ReplicationSlotStatus {
    ReplicationSlotStatus {
        slot_name: "trellara_slot".to_string(),
        exists,
        plugin: plugin.map(ToString::to_string),
        expected_plugin: "pgoutput".to_string(),
        active: None,
        failover: None,
        synced: None,
        inactive_since: None,
        idle_replication_slot_timeout: None,
        restart_lsn: None,
        confirmed_flush_lsn: None,
        retained_wal_bytes: None,
        wal_status: None,
        safe_wal_size_bytes: None,
        invalidation_reason: None,
        wal_headroom: None,
        transaction_id_wraparound: None,
        xmin_horizon: None,
        issues: Vec::new(),
    }
}
