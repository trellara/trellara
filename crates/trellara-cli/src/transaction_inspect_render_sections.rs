use std::fmt::Write as _;

use crate::{is_strict_chunked_mode, TransactionInspectSummary};

pub(crate) fn push_ddl_events(output: &mut String, summary: &TransactionInspectSummary) {
    output.push_str("\nddl_events:\n");
    if summary.ddl_events.is_empty() {
        output.push_str("- none\n");
    } else {
        for event in &summary.ddl_events {
            writeln!(
                output,
                "- order={} operation={} relation={} auto_apply={} release_gate={}",
                event.total_order,
                event.operation,
                event.relation,
                event.target_auto_apply,
                empty_label(&event.release_gate)
            )
            .expect("write string");
        }
    }
    if let Some(replay) = &summary.dml_replay_after_ddl_barrier {
        writeln!(
            output,
            "dml_replay_after_ddl_barrier: barrier_id={} changes={} checksum={} ddl_events_stripped={} reason={}",
            replay.barrier_id,
            replay.change_count,
            replay.checksum,
            replay.ddl_events_stripped,
            replay.reason
        )
        .expect("write string");
    }
}

pub(crate) fn push_affected_tables(output: &mut String, summary: &TransactionInspectSummary) {
    output.push_str("\naffected_tables:\n");
    if summary.affected_tables.is_empty() {
        output.push_str("- none\n");
        return;
    }
    for table in &summary.affected_tables {
        writeln!(
            output,
            "- {} events={} inserts={} updates={} deletes={} truncates={}",
            table.relation,
            table.event_count,
            table.inserts,
            table.updates,
            table.deletes,
            table.truncates
        )
        .expect("write string");
    }
}

pub(crate) fn push_manifest(output: &mut String, summary: &TransactionInspectSummary) {
    if let Some(manifest) = &summary.partition_manifest {
        let (section_label, unit_label, count_label) =
            manifest_inspect_labels(&manifest.boundary_mode);
        writeln!(output, "\n{section_label}:").expect("write string");
        writeln!(
            output,
            "boundary_mode={} global_event_count={} {}={} checksum={} source_commit_lsn={}",
            manifest.boundary_mode,
            manifest.global_event_count,
            count_label,
            manifest.participating_partition_count,
            manifest.checksum,
            manifest.source_commit_lsn
        )
        .expect("write string");
        for partition in &manifest.partitions {
            writeln!(
                output,
                "- {}={} events={} total_order={}..{} checksum={}",
                unit_label,
                partition.id,
                partition.event_count,
                partition.first_total_order,
                partition.last_total_order,
                partition.checksum
            )
            .expect("write string");
        }
    }
}

fn empty_label(value: &str) -> &str {
    if value.is_empty() {
        "none"
    } else {
        value
    }
}

fn manifest_inspect_labels(boundary_mode: &str) -> (&'static str, &'static str, &'static str) {
    if is_strict_chunked_mode(boundary_mode) {
        (
            "strict_chunk_manifest",
            "chunk",
            "participating_chunk_count",
        )
    } else {
        (
            "partition_manifest",
            "partition",
            "participating_partition_count",
        )
    }
}
