use std::fmt::Write as _;

use trellara_checkpoint::{DdlBarrierReleaseBlocker, DdlBarrierSummary};

use crate::list_or_none;

pub(crate) fn push_next_actions(output: &mut String, summary: &DdlBarrierSummary) {
    writeln!(output, "next_actions:").expect("write string");
    let actions = next_actions(summary);
    if actions.is_empty() {
        writeln!(output, "- none").expect("write string");
    } else {
        for action in actions {
            writeln!(output, "- {action}").expect("write string");
        }
    }
}

pub(crate) fn next_actions(summary: &DdlBarrierSummary) -> Vec<String> {
    let mut actions = Vec::new();
    if !summary.pending_sinks.is_empty() {
        push_action(&mut actions, format!(
            "record required ACKs for pending sinks {} on barrier {} with ack_lsn at or beyond {} and schema_version {}",
            list_or_none(&summary.pending_sinks),
            summary.barrier_id,
            summary.barrier_lsn,
            summary.schema_version
        ));
    }
    if !summary.unexpected_sinks.is_empty() {
        push_action(
            &mut actions,
            format!(
                "remove or investigate unexpected sink ACKs before release: {}",
                list_or_none(&summary.unexpected_sinks)
            ),
        );
    }
    push_detail_actions(&mut actions, summary);
    if has_code(summary, "insufficient_target_postgres_evidence") {
        push_action(&mut actions,
            "rerun target_postgres DDL apply and record ACK detail containing applied DDL statements, plan_sha256, statement_sha256 values, and release_gate=post_ddl_dml_release"
                .to_string(),
        );
    }
    if has_code(summary, "insufficient_raw_cdc_lake_evidence") {
        push_action(&mut actions, format!(
            "rerun raw CDC lake epoch finalization for barrier {} and record raw_cdc_lake ACK detail containing schema_version {}, epoch_id, metadata_table, partition_metadata_table, manifest_digest, and release_gate=post_ddl_dml_release",
            summary.barrier_id, summary.schema_version
        ));
    }
    if has_code(summary, "insufficient_spark_derived_views_evidence") {
        push_action(&mut actions,
            "regenerate Spark derived view templates and record spark_derived_views ACK detail containing regenerated view_count, template_digest, accepted_by, and release_gate=post_ddl_dml_release"
                .to_string(),
        );
    }
    if has_code(summary, "schema_version_mismatch") {
        push_action(
            &mut actions,
            "refresh schema discovery and record ACKs with the barrier schema_version".to_string(),
        );
    }
    if has_code(summary, "ack_lsn_before_barrier") {
        push_action(
            &mut actions,
            format!(
                "wait for sinks to reach barrier_lsn {}, then record a fresh ACK for barrier {}",
                summary.barrier_lsn, summary.barrier_id
            ),
        );
    }
    if has_code(summary, "partition_visibility_not_released") {
        push_action(&mut actions, format!(
            "run partition-watermarks for barrier {}, verify every partition lane reached barrier_lsn {}, then record the partition_visibility ACK with --barrier-lsn {} --schema-version {}, --partition-durable-lsn for each lane, and --partition-applied-lsn for each lane",
            summary.barrier_id,
            summary.barrier_lsn,
            summary.barrier_lsn,
            summary.schema_version
        ));
    }
    actions
}

fn push_detail_actions(actions: &mut Vec<String>, summary: &DdlBarrierSummary) {
    for blocker in &summary.release_blocker_details {
        match blocker.code.as_str() {
            "pending_required_ack" => push_action(
                actions,
                format!(
                    "capture durable sink ACK evidence for {} using {}",
                    blocker_sinks(blocker),
                    blocker.evidence
                ),
            ),
            "rejected_or_stale_ack" => push_action(
                actions,
                format!(
                    "replace rejected or stale ACK evidence for {} before releasing post-DDL DML; blocker evidence: {}",
                    blocker_sinks(blocker),
                    blocker.evidence
                ),
            ),
            "unexpected_ack" | "unexpected_sink_ack" => push_action(
                actions,
                format!(
                    "remove ACK records for non-required sinks {} or add them to the barrier required_sinks contract; blocker evidence: {}",
                    blocker_sinks(blocker),
                    blocker.evidence
                ),
            ),
            "missing_required_sinks" => push_action(
                actions,
                format!(
                    "define the barrier required_sinks contract before release; blocker evidence: {}",
                    blocker.evidence
                ),
            ),
            _ => {}
        }
    }
}

fn blocker_sinks(blocker: &DdlBarrierReleaseBlocker) -> String {
    list_or_none(&blocker.sinks)
}

fn has_code(summary: &DdlBarrierSummary, code: &str) -> bool {
    summary
        .release_blocker_codes
        .iter()
        .any(|item| item == code)
        || summary
            .release_blocker_details
            .iter()
            .any(|blocker| blocker.code == code)
}

fn push_action(actions: &mut Vec<String>, action: String) {
    if !actions.iter().any(|existing| existing == &action) {
        actions.push(action);
    }
}
