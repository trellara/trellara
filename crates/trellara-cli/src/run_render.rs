use std::fmt::Write as _;

use crate::{checksum_status_label, run_proof_status_label, RunSummary};

pub(crate) fn render_run_summary_text(summary: &RunSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara local run").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "mode: {}", summary.mode).expect("write string");
    writeln!(&mut output, "stream: {}", summary.stream_kind).expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "phase_summary:").expect("write string");
    writeln!(
        &mut output,
        "- bootstrap: publication={} slot={} consistent_lsn={}",
        summary.bootstrap.publication,
        summary.bootstrap.slot,
        summary
            .bootstrap
            .consistent_lsn
            .as_deref()
            .unwrap_or("missing")
    )
    .expect("write string");
    if let Some(snapshot) = &summary.snapshot {
        writeln!(
            &mut output,
            "- snapshot: run={} state={} copied_rows={} consistent_lsn={}",
            snapshot.run_id, snapshot.state, snapshot.copied_rows, snapshot.consistent_lsn
        )
        .expect("write string");
    } else {
        writeln!(&mut output, "- snapshot: skipped").expect("write string");
    }
    writeln!(
        &mut output,
        "- relay: {} transactions, {} messages, last_lsn={}",
        summary.relay.published_transactions,
        summary.relay.published_messages,
        summary.relay.last_commit_lsn.as_deref().unwrap_or("none")
    )
    .expect("write string");
    writeln!(
        &mut output,
        "- apply: {} transactions, {} changes, {} acked_messages, last_lsn={}",
        summary.apply.applied_transactions,
        summary.apply.applied_changes,
        summary.apply.acked_messages,
        summary.apply.last_commit_lsn.as_deref().unwrap_or("none")
    )
    .expect("write string");
    if summary.apply.barrier_pending_transactions > 0 {
        writeln!(
            &mut output,
            "  barrier_pending: transactions={} missing_manifest={} missing_commit_marker={} invalid_commit_marker={} missing_chunks={} extra_chunks={} buffered_chunks={} buffered_messages={}",
            summary.apply.barrier_pending_transactions,
            summary.apply.barrier_pending_missing_manifest,
            summary.apply.barrier_pending_missing_commit_marker,
            summary.apply.barrier_pending_invalid_commit_marker,
            summary.apply.barrier_pending_missing_chunks,
            summary.apply.barrier_pending_extra_chunks,
            summary.apply.barrier_pending_buffered_chunks,
            summary.apply.barrier_pending_buffered_messages
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  barrier_pending_blockers: {}",
            list_or_none(&summary.apply.barrier_pending.blockers)
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  barrier_pending_blocker_codes: {}",
            list_or_none(&summary.apply.barrier_pending.blocker_codes)
        )
        .expect("write string");
        writeln!(
            &mut output,
            "  barrier_pending_recovery_actions: {}",
            list_or_none(&summary.apply.barrier_pending.recovery_actions)
        )
        .expect("write string");
    }
    if let Some(verify) = &summary.verify {
        writeln!(
            &mut output,
            "- verify: converged={} checksum_status={}",
            verify.converged,
            checksum_status_label(verify.checksum_status)
        )
        .expect("write string");
    } else {
        writeln!(&mut output, "- verify: not run").expect("write string");
    }

    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "proof_chain:").expect("write string");
    for gate in &summary.proof_chain {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            run_proof_status_label(gate.status),
            gate.code,
            gate.evidence
        )
        .expect("write string");
        writeln!(&mut output, "  proof: {}", gate.proof_command).expect("write string");
    }

    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "next_commands:").expect("write string");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}

fn list_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
