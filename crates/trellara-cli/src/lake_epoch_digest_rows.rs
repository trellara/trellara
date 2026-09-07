use crate::{
    LakeEpochPartitionRollup, LakeEpochQuarantineEntry, LakeEpochSourceWatermark,
    LakeEpochTableRollup,
};

use super::lake_epoch_digest_manifest::push_manifest_line;

pub(super) fn push_source_lines(manifest: &mut String, sources: &[LakeEpochSourceWatermark]) {
    let mut sources = sources.to_vec();
    sources.sort_by(|left, right| left.source_id.cmp(&right.source_id));

    for source in &sources {
        push_manifest_line(
            manifest,
            "source",
            &format!(
                "{}|{}|{}|{}|{}|{}|{}",
                source.source_id,
                source.state,
                source.start_lsn.as_deref().unwrap_or(""),
                source.end_lsn.as_deref().unwrap_or(""),
                source.transaction_count,
                source.change_count,
                source.gap_reason.as_deref().unwrap_or("")
            ),
        );
    }
}

pub(super) fn push_table_lines(manifest: &mut String, tables: &[LakeEpochTableRollup]) {
    let mut tables = tables.to_vec();
    tables.sort_by(|left, right| left.relation.cmp(&right.relation));

    for table in &tables {
        push_manifest_line(
            manifest,
            "table",
            &format!(
                "{}|{}|{}|{}",
                table.relation, table.transaction_count, table.change_count, table.checksum_rollup
            ),
        );
    }
}

pub(super) fn push_partition_lines(manifest: &mut String, partitions: &[LakeEpochPartitionRollup]) {
    let mut partitions = partitions.to_vec();
    partitions.sort_by(|left, right| {
        (&left.source_id, left.partition_id).cmp(&(&right.source_id, right.partition_id))
    });

    for partition in &partitions {
        push_manifest_line(
            manifest,
            "partition",
            &format!(
                "{}|{}|{}|{}|{}|{}|{}",
                partition.source_id,
                partition.partition_id,
                partition.first_commit_lsn.as_deref().unwrap_or(""),
                partition.last_commit_lsn.as_deref().unwrap_or(""),
                partition.transaction_count,
                partition.event_count,
                partition.checksum_rollup
            ),
        );
    }
}

pub(super) fn push_quarantine_lines(
    manifest: &mut String,
    quarantine_entries: &[LakeEpochQuarantineEntry],
) {
    let mut quarantine_entries = quarantine_entries.to_vec();
    quarantine_entries.sort_by(|left, right| {
        (
            &left.source_id,
            &left.transaction_id,
            &left.commit_lsn,
            &left.reason,
        )
            .cmp(&(
                &right.source_id,
                &right.transaction_id,
                &right.commit_lsn,
                &right.reason,
            ))
    });

    for entry in &quarantine_entries {
        push_manifest_line(
            manifest,
            "quarantine",
            &format!(
                "{}|{}|{}|{}|{}|{}",
                entry.source_id,
                entry.transaction_id.as_deref().unwrap_or(""),
                entry.commit_lsn.as_deref().unwrap_or(""),
                entry.reason,
                entry.details,
                entry.recovery_command
            ),
        );
    }
}
