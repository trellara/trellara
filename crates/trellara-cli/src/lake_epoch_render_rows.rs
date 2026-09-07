use std::fmt::Write as _;

use crate::LakeEpochSummary;

pub(crate) fn push_source_watermark_rows(output: &mut String, summary: &LakeEpochSummary) {
    writeln!(output, "source_watermarks:").expect("write string");
    for source in &summary.source_watermarks {
        writeln!(
            output,
            "- {} state={} start_lsn={} end_lsn={} transactions={} changes={} gap_reason={}",
            source.source_id,
            source.state,
            source.start_lsn.as_deref().unwrap_or("none"),
            source.end_lsn.as_deref().unwrap_or("none"),
            source.transaction_count,
            source.change_count,
            source.gap_reason.as_deref().unwrap_or("none")
        )
        .expect("write string");
    }
}

pub(crate) fn push_table_rollup_rows(output: &mut String, summary: &LakeEpochSummary) {
    writeln!(output, "table_rollups:").expect("write string");
    for table in &summary.table_rollups {
        writeln!(
            output,
            "- {} transactions={} changes={} checksum_rollup={}",
            table.relation, table.transaction_count, table.change_count, table.checksum_rollup
        )
        .expect("write string");
    }
}

pub(crate) fn push_partition_rollup_rows(output: &mut String, summary: &LakeEpochSummary) {
    writeln!(output, "partition_rollups:").expect("write string");
    for partition in &summary.partition_rollups {
        writeln!(
            output,
            "- source={} partition={} first_commit_lsn={} last_commit_lsn={} transactions={} events={} checksum_rollup={}",
            partition.source_id,
            partition.partition_id,
            partition.first_commit_lsn.as_deref().unwrap_or("none"),
            partition.last_commit_lsn.as_deref().unwrap_or("none"),
            partition.transaction_count,
            partition.event_count,
            partition.checksum_rollup
        )
        .expect("write string");
    }
}

pub(crate) fn push_quarantine_entry_rows(output: &mut String, summary: &LakeEpochSummary) {
    writeln!(output, "quarantine_entries:").expect("write string");
    for entry in &summary.quarantine_entries {
        writeln!(
            output,
            "- source={} transaction_id={} commit_lsn={} reason={} recovery_command={}",
            entry.source_id,
            entry.transaction_id.as_deref().unwrap_or("none"),
            entry.commit_lsn.as_deref().unwrap_or("none"),
            entry.reason,
            entry.recovery_command
        )
        .expect("write string");
    }
}
