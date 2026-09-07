use trellara_pg_capture::ReplicationSlotStatus;

use crate::wal_headroom_label;

pub(crate) fn source_slot_position_evidence(source_slot: &ReplicationSlotStatus) -> String {
    let mut fields = vec![format!(
        "restart_lsn={} confirmed_flush_lsn={} wal_status={} safe_wal_size_bytes={} retained_wal_bytes={}",
        source_slot
            .restart_lsn
            .as_deref()
            .filter(|lsn| !lsn.trim().is_empty())
            .unwrap_or("unknown"),
        source_slot
            .confirmed_flush_lsn
            .as_deref()
            .filter(|lsn| !lsn.trim().is_empty())
            .unwrap_or("unknown"),
        source_slot
            .wal_status
            .as_deref()
            .filter(|status| !status.trim().is_empty())
            .unwrap_or("unknown"),
        source_slot
            .safe_wal_size_bytes
            .map(|bytes| bytes.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        source_slot
            .retained_wal_bytes
            .map(|bytes| bytes.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    )];

    push_optional_field(
        &mut fields,
        "invalidation_reason",
        source_slot.invalidation_reason.as_deref(),
    );
    push_optional_bool_field(&mut fields, "failover", source_slot.failover);
    push_optional_bool_field(&mut fields, "synced", source_slot.synced);
    push_optional_field(
        &mut fields,
        "inactive_since",
        source_slot.inactive_since.as_deref(),
    );
    push_optional_field(
        &mut fields,
        "idle_replication_slot_timeout",
        source_slot.idle_replication_slot_timeout.as_deref(),
    );
    if let Some(headroom) = wal_headroom_label(source_slot) {
        fields.push(format!("wal_headroom={headroom}"));
    }

    fields.join(" ")
}

fn push_optional_field(fields: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        fields.push(format!("{name}={value}"));
    }
}

fn push_optional_bool_field(fields: &mut Vec<String>, name: &str, value: Option<bool>) {
    if let Some(value) = value {
        fields.push(format!("{name}={value}"));
    }
}
