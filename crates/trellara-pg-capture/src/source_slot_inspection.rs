use crate::{CaptureError, ReplicationSlotStatus, Result};

pub(crate) fn fail_on_slot_plugin_mismatch(status: &ReplicationSlotStatus) -> Result<()> {
    if !status.exists {
        return Ok(());
    }
    let actual_plugin = status.plugin.as_deref().unwrap_or("unknown");
    if actual_plugin == status.expected_plugin {
        return Ok(());
    }
    Err(CaptureError::SlotPluginMismatch {
        slot_name: status.slot_name.clone(),
        expected_plugin: status.expected_plugin.clone(),
        actual_plugin: actual_plugin.to_string(),
    })
}

pub(crate) fn replication_slot_status_from_row(
    row: tokio_postgres::Row,
    expected_plugin: &str,
) -> ReplicationSlotStatus {
    replication_slot_status_from_row_with_expected(row, Some(expected_plugin))
}

pub(crate) fn observed_replication_slot_status_from_row(
    row: tokio_postgres::Row,
) -> ReplicationSlotStatus {
    replication_slot_status_from_row_with_expected(row, None)
}

fn replication_slot_status_from_row_with_expected(
    row: tokio_postgres::Row,
    expected_plugin: Option<&str>,
) -> ReplicationSlotStatus {
    let slot_name = row.get::<_, String>("slot_name");
    let plugin = row.get::<_, Option<String>>("plugin");
    let exists = plugin.is_some();
    let expected_plugin = expected_plugin
        .or(plugin.as_deref())
        .unwrap_or("unknown")
        .to_string();
    let active = row.get::<_, Option<bool>>("active");
    let failover = row.get::<_, Option<bool>>("failover");
    let synced = row.get::<_, Option<bool>>("synced");
    let inactive_since = row.get::<_, Option<String>>("inactive_since");
    let idle_replication_slot_timeout =
        row.get::<_, Option<String>>("idle_replication_slot_timeout");
    let restart_lsn = row.get::<_, Option<String>>("restart_lsn");
    let confirmed_flush_lsn = row.get::<_, Option<String>>("confirmed_flush_lsn");
    let retained_wal_bytes = row.get::<_, Option<i64>>("retained_wal_bytes");
    let wal_status = row.get::<_, Option<String>>("wal_status");
    let safe_wal_size_bytes = row.get::<_, Option<i64>>("safe_wal_size_bytes");
    let invalidation_reason = row.get::<_, Option<String>>("invalidation_reason");
    let issues = slot_status_issues(
        exists,
        plugin.as_deref(),
        &expected_plugin,
        retained_wal_bytes,
        wal_status.as_deref(),
        safe_wal_size_bytes,
        invalidation_reason.as_deref(),
    );

    ReplicationSlotStatus {
        slot_name,
        exists,
        plugin,
        expected_plugin,
        active,
        failover,
        synced,
        inactive_since,
        idle_replication_slot_timeout,
        restart_lsn,
        confirmed_flush_lsn,
        retained_wal_bytes,
        wal_status,
        safe_wal_size_bytes,
        invalidation_reason,
        wal_headroom: None,
        transaction_id_wraparound: None,
        xmin_horizon: None,
        issues,
    }
}

pub(crate) fn slot_status_issues(
    exists: bool,
    plugin: Option<&str>,
    expected_plugin: &str,
    retained_wal_bytes: Option<i64>,
    wal_status: Option<&str>,
    safe_wal_size_bytes: Option<i64>,
    invalidation_reason: Option<&str>,
) -> Vec<String> {
    let mut issues = Vec::new();
    if !exists {
        issues.push("replication slot does not exist".to_string());
        return issues;
    }
    if plugin != Some(expected_plugin) {
        issues.push(format!(
            "replication slot plugin is {}, expected {}",
            plugin.unwrap_or("unknown"),
            expected_plugin
        ));
    }
    if let Some(bytes) = retained_wal_bytes {
        if bytes < 0 {
            issues.push("replication slot restart_lsn is ahead of current WAL".to_string());
        }
    }
    if let Some(reason) = invalidation_reason.filter(|reason| !reason.trim().is_empty()) {
        issues.push(format!(
            "replication slot invalidated: {reason}; recreate the slot and reseed affected targets"
        ));
    }
    match wal_status {
        Some("lost") => issues.push(
            "replication slot WAL is lost; recreate the slot and reseed affected targets"
                .to_string(),
        ),
        Some("unreserved") => issues.push(
            "replication slot WAL is unreserved and at risk of loss; drain or reseed before continuing"
                .to_string(),
        ),
        _ => {}
    }
    if let Some(bytes) = safe_wal_size_bytes {
        if bytes <= 0 {
            issues.push(
                "replication slot safe WAL size is exhausted; drain or reseed before continuing"
                    .to_string(),
            );
        }
    }
    issues
}
