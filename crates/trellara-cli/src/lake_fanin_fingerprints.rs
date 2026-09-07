use crate::LakeEpochSummary;

pub(super) fn lake_epoch_watermark_rollup_fingerprint(epoch: &LakeEpochSummary) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        epoch
            .watermark_rollup
            .global_low_watermark_lsn
            .as_deref()
            .unwrap_or(""),
        epoch
            .watermark_rollup
            .max_source_watermark_lsn
            .as_deref()
            .unwrap_or(""),
        epoch.watermark_rollup.complete_source_count,
        epoch.watermark_rollup.lagging_source_count,
        epoch.watermark_rollup.missing_source_count,
        epoch.watermark_rollup.quarantined_source_count,
        epoch.watermark_rollup.invalid_lsn_source_count,
        epoch.watermark_rollup.invalid_lsn_sources.join(",")
    )
}

pub(super) fn lake_epoch_source_watermark_fingerprint(epoch: &LakeEpochSummary) -> String {
    sorted_fingerprint(epoch.source_watermarks.iter().map(|source| {
        format!(
            "{}:{}:{}:{}:{}:{}:{}",
            source.source_id,
            source.state,
            source.start_lsn.as_deref().unwrap_or(""),
            source.end_lsn.as_deref().unwrap_or(""),
            source.transaction_count,
            source.change_count,
            source.gap_reason.as_deref().unwrap_or("")
        )
    }))
}

pub(super) fn lake_epoch_quarantine_fingerprint(epoch: &LakeEpochSummary) -> String {
    sorted_fingerprint(epoch.quarantine_entries.iter().map(|entry| {
        format!(
            "{}:{}:{}:{}:{}:{}",
            entry.source_id,
            entry.transaction_id.as_deref().unwrap_or(""),
            entry.commit_lsn.as_deref().unwrap_or(""),
            entry.reason,
            entry.details,
            entry.recovery_command
        )
    }))
}

pub(super) fn lake_epoch_table_rollup_fingerprint(epoch: &LakeEpochSummary) -> String {
    sorted_fingerprint(epoch.table_rollups.iter().map(|table| {
        format!(
            "{}:{}:{}:{}",
            table.relation, table.transaction_count, table.change_count, table.checksum_rollup
        )
    }))
}

pub(super) fn lake_epoch_partition_rollup_fingerprint(epoch: &LakeEpochSummary) -> String {
    sorted_fingerprint(epoch.partition_rollups.iter().map(|partition| {
        format!(
            "{}:{}:{}:{}:{}:{}:{}",
            partition.source_id,
            partition.partition_id,
            partition.first_commit_lsn.as_deref().unwrap_or(""),
            partition.last_commit_lsn.as_deref().unwrap_or(""),
            partition.transaction_count,
            partition.event_count,
            partition.checksum_rollup
        )
    }))
}

fn sorted_fingerprint(parts: impl Iterator<Item = String>) -> String {
    let mut parts = parts.collect::<Vec<_>>();
    parts.sort_unstable();
    parts.join("|")
}
