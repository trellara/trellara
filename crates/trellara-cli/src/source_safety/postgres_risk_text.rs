use trellara_pg_capture::ReplicationSlotStatus;

pub(crate) fn wal_headroom_label(source_slot: &ReplicationSlotStatus) -> Option<String> {
    source_slot.wal_headroom.as_ref().map(|projection| {
        let headroom = projection
            .headroom_seconds
            .map(format_duration)
            .unwrap_or_else(|| "not burning down during sample".to_string());
        format!(
            "{headroom} at {} bytes/sec over {}ms",
            projection.wal_bytes_per_second, projection.sample_ms
        )
    })
}

pub(crate) fn format_duration(seconds: i64) -> String {
    let seconds = seconds.max(0);
    if seconds >= 60 * 60 {
        format!("~{} hours", ((seconds + 30 * 60) / (60 * 60)).max(1))
    } else if seconds >= 60 {
        format!("~{} minutes", ((seconds + 30) / 60).max(1))
    } else {
        format!("~{} seconds", seconds)
    }
}

pub(crate) fn display_optional_duration(seconds: Option<i64>) -> String {
    seconds
        .map(format_duration)
        .unwrap_or_else(|| "none".to_string())
}

pub(crate) fn display_optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
