use std::fmt::Write as _;

use crate::TransactionInspectBoundaryProof;

pub(crate) fn push_boundary_proof(output: &mut String, boundary: &TransactionInspectBoundaryProof) {
    output.push_str("\nboundary_proof:\n");
    writeln!(
        output,
        "- manifest_barrier_required={}",
        boundary.manifest_barrier_required
    )
    .expect("write string");
    writeln!(output, "- manifest_valid={}", boundary.manifest_valid).expect("write string");
    if let Some(error) = &boundary.manifest_validation_error {
        writeln!(output, "- manifest_validation_error={error}").expect("write string");
    }
    writeln!(
        output,
        "- source_boundary_kind={}",
        boundary.source_boundary_kind
    )
    .expect("write string");
    writeln!(
        output,
        "- partitioned_scale_decision={}",
        boundary.partitioned_scale_decision
    )
    .expect("write string");
    writeln!(
        output,
        "- partition_parallel_safe={}",
        boundary.partition_parallel_safe
    )
    .expect("write string");
    writeln!(
        output,
        "- requires_ddl_barrier={}",
        boundary.requires_ddl_barrier
    )
    .expect("write string");
    writeln!(
        output,
        "- dml_replay_after_ddl_barrier_required={}",
        boundary.dml_replay_after_ddl_barrier_required
    )
    .expect("write string");
    writeln!(
        output,
        "- partitioned_scale_reason={}",
        boundary.partitioned_scale_reason
    )
    .expect("write string");
    push_manifest_evidence(output, boundary);
    writeln!(
        output,
        "- participating_partition_count={}",
        boundary.participating_partition_count
    )
    .expect("write string");
    if let Some(checksum) = boundary.expected_commit_marker_manifest_checksum {
        writeln!(
            output,
            "- expected_commit_marker_manifest_checksum={checksum}"
        )
        .expect("write string");
    }
}

fn push_manifest_evidence(output: &mut String, boundary: &TransactionInspectBoundaryProof) {
    if let Some(matches) = boundary.global_event_count_matches {
        writeln!(output, "- global_event_count_matches={matches}").expect("write string");
    }
    if let Some(matches) = boundary.partition_event_count_matches {
        writeln!(output, "- partition_event_count_matches={matches}").expect("write string");
    }
    if let Some(checksum) = boundary.partitioned_scale_manifest_checksum {
        writeln!(output, "- partitioned_scale_manifest_checksum={checksum}").expect("write string");
    }
    if let Some(count) = boundary.partitioned_scale_manifest_event_count {
        writeln!(output, "- partitioned_scale_manifest_event_count={count}").expect("write string");
    }
    if let Some(count) = boundary.partitioned_scale_envelope_event_count {
        writeln!(output, "- partitioned_scale_envelope_event_count={count}").expect("write string");
    }
    if let Some(covered) = boundary.partitioned_scale_event_count_coverage {
        writeln!(output, "- partitioned_scale_event_count_coverage={covered}")
            .expect("write string");
    }
    push_partition_ids(output, boundary);
    if let Some(contract) = &boundary.partitioned_scale_visibility_contract {
        writeln!(output, "- partitioned_scale_visibility_contract={contract}")
            .expect("write string");
    }
}

fn push_partition_ids(output: &mut String, boundary: &TransactionInspectBoundaryProof) {
    if boundary
        .partitioned_scale_participating_partition_ids
        .is_empty()
    {
        return;
    }
    writeln!(
        output,
        "- partitioned_scale_participating_partition_ids={}",
        boundary
            .partitioned_scale_participating_partition_ids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
    .expect("write string");
}
