use trellara_pg_capture::ReplicationSlotStatus;

use crate::{optional_bool_label, wal_headroom_label};

pub(crate) fn source_slot_summary_line(slot: &ReplicationSlotStatus) -> String {
    let mut fields = vec![
        format!("slot: {}", slot.slot_name),
        format!("plugin={}", slot.plugin.as_deref().unwrap_or("unknown")),
        format!("active={}", optional_bool_label(slot.active)),
        format!(
            "restart_lsn={}",
            non_empty_or_unknown(slot.restart_lsn.as_deref())
        ),
        format!(
            "confirmed_flush_lsn={}",
            non_empty_or_unknown(slot.confirmed_flush_lsn.as_deref())
        ),
        format!(
            "wal_status={}",
            non_empty_or_unknown(slot.wal_status.as_deref())
        ),
        format!(
            "safe_wal_size_bytes={}",
            slot.safe_wal_size_bytes
                .map(|bytes| bytes.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ),
        format!(
            "retained_wal_bytes={}",
            slot.retained_wal_bytes
                .map(|bytes| bytes.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        ),
    ];

    push_optional_slot_field(
        &mut fields,
        "invalidation_reason",
        slot.invalidation_reason.as_deref(),
    );
    push_optional_slot_bool_field(&mut fields, "failover", slot.failover);
    push_optional_slot_bool_field(&mut fields, "synced", slot.synced);
    push_optional_slot_field(
        &mut fields,
        "inactive_since",
        slot.inactive_since.as_deref(),
    );
    push_optional_slot_field(
        &mut fields,
        "idle_replication_slot_timeout",
        slot.idle_replication_slot_timeout.as_deref(),
    );
    if let Some(headroom) = wal_headroom_label(slot) {
        fields.push(format!("wal_headroom={headroom}"));
    }

    fields.join(" ")
}

fn non_empty_or_unknown(value: Option<&str>) -> &str {
    value
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unknown")
}

fn push_optional_slot_field(fields: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        fields.push(format!("{name}={value}"));
    }
}

fn push_optional_slot_bool_field(fields: &mut Vec<String>, name: &str, value: Option<bool>) {
    if let Some(value) = value {
        fields.push(format!("{name}={value}"));
    }
}
