use std::fmt::Write as _;

use trellara_checkpoint::DdlBarrierSummary;

use crate::list_or_none;

pub(crate) fn push_barrier_header(output: &mut String, summary: &DdlBarrierSummary) {
    writeln!(output, "Trellara DDL barrier status").expect("write string");
    writeln!(
        output,
        "dataset: {} source={} database={}",
        summary.dataset_id, summary.source_id, summary.database_id
    )
    .expect("write string");
    writeln!(
        output,
        "barrier: {} lsn={} schema_version={}",
        summary.barrier_id, summary.barrier_lsn, summary.schema_version
    )
    .expect("write string");
    writeln!(
        output,
        "cdc_transaction_boundary: {}",
        summary.cdc_transaction_boundary
    )
    .expect("write string");
    if let Some(boundary) = &summary.propagation_boundary {
        writeln!(output, "propagation_boundary: {boundary}").expect("write string");
    }
    if !summary.propagation_decisions.is_empty() {
        writeln!(
            output,
            "propagation_decisions: {}",
            list_or_none(&summary.propagation_decisions)
        )
        .expect("write string");
    }
    if let Some(digest) = &summary.propagation_policy_sha256 {
        writeln!(output, "propagation_policy_sha256: {digest}").expect("write string");
    }
}

pub(crate) fn push_barrier_sink_summary(output: &mut String, summary: &DdlBarrierSummary) {
    writeln!(
        output,
        "release_dml: {} required_sinks={} acked={} pending={} rejected={} unexpected={} partition_pause={}",
        summary.release_dml,
        summary.required_sink_count,
        summary.acked_sink_count,
        summary.pending_sink_count,
        summary.rejected_sink_count,
        summary.unexpected_ack_count,
        summary.requires_global_partition_pause
    )
    .expect("write string");
    writeln!(
        output,
        "acked_sinks: {}",
        list_or_none(&summary.acked_sinks)
    )
    .expect("write string");
    writeln!(
        output,
        "pending_sinks: {}",
        list_or_none(&summary.pending_sinks)
    )
    .expect("write string");
    writeln!(
        output,
        "rejected_sinks: {}",
        list_or_none(&summary.rejected_sinks)
    )
    .expect("write string");
    writeln!(
        output,
        "unexpected_sinks: {} count={}",
        list_or_none(&summary.unexpected_sinks),
        summary.unexpected_ack_count
    )
    .expect("write string");
}

pub(crate) fn push_release_summary(output: &mut String, summary: &DdlBarrierSummary) {
    writeln!(
        output,
        "release_blockers: {}",
        list_or_none(&summary.release_blockers)
    )
    .expect("write string");
    writeln!(
        output,
        "release_blocker_codes: {}",
        list_or_none(&summary.release_blocker_codes)
    )
    .expect("write string");
}

pub(crate) fn push_release_blocker_details(output: &mut String, summary: &DdlBarrierSummary) {
    writeln!(output, "release_blocker_details:").expect("write string");
    if summary.release_blocker_details.is_empty() {
        writeln!(output, "- none").expect("write string");
        return;
    }
    for blocker in &summary.release_blocker_details {
        writeln!(
            output,
            "- code={} sinks={} message={} evidence={}",
            blocker.code,
            list_or_none(&blocker.sinks),
            blocker.message,
            blocker.evidence
        )
        .expect("write string");
    }
}

pub(crate) fn push_release_gates(output: &mut String, summary: &DdlBarrierSummary) {
    writeln!(output, "release_gates:").expect("write string");
    for gate in &summary.release_gates {
        writeln!(
            output,
            "- {} satisfied={} evidence={}",
            gate.name, gate.satisfied, gate.evidence
        )
        .expect("write string");
    }
}

pub(crate) fn push_sink_evidence(output: &mut String, summary: &DdlBarrierSummary) {
    writeln!(output, "sink_evidence:").expect("write string");
    for evidence in &summary.sink_evidence {
        writeln!(
            output,
            "- {} status={} release_eligible={} ack_lsn={} schema_version={} accepted={} rejection_code={} reason={}",
            evidence.sink,
            evidence.status,
            evidence.release_eligible,
            evidence.ack_lsn.as_deref().unwrap_or("none"),
            evidence.schema_version.as_deref().unwrap_or("none"),
            evidence
                .accepted
                .map(|accepted| accepted.to_string())
                .unwrap_or_else(|| "none".to_string()),
            evidence.rejection_code.as_deref().unwrap_or("none"),
            evidence.rejection_reason.as_deref().unwrap_or("none")
        )
        .expect("write string");
        if let Some(detail) = &evidence.detail {
            writeln!(output, "  detail: {detail}").expect("write string");
        }
    }
}
