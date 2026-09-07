use std::fmt::Write as _;

use crate::{
    lake_completeness_state_label, LakeFaninVerifyMismatchSeverity, LakeFaninVerifyStatus,
    LakeFaninVerifySummary, QuickstartOutputFormat, Result,
};

pub(crate) fn render_lake_fanin_verify_summary(
    summary: &LakeFaninVerifySummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_lake_fanin_verify_text(summary)),
    }
}

fn render_lake_fanin_verify_text(summary: &LakeFaninVerifySummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara lake fan-in verification").expect("write string");
    writeln!(
        &mut output,
        "dataset: {} source={} mode={}",
        summary.dataset_id, summary.source_id, summary.mode
    )
    .expect("write string");
    writeln!(
        &mut output,
        "epoch: {} status={} spark_consumption_allowed={}",
        summary.epoch_id,
        lake_fanin_verify_status_label(summary.status),
        summary.spark_consumption_allowed
    )
    .expect("write string");
    writeln!(
        &mut output,
        "spark_consumption_gate: {}",
        summary.spark_consumption_gate
    )
    .expect("write string");
    writeln!(
        &mut output,
        "spark_consumption_contract: {}",
        summary.spark_consumption_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "states: stream={} lake={}",
        lake_completeness_state_label(summary.stream_state),
        lake_completeness_state_label(summary.lake_state)
    )
    .expect("write string");
    push_count_lines(&mut output, summary);
    if !summary.mismatches.is_empty() {
        push_mismatches(&mut output, summary);
    }
    writeln!(&mut output, "proof: {}", summary.proof_command).expect("write string");
    writeln!(&mut output, "next_steps:").expect("write string");
    for step in &summary.recommended_next_steps {
        writeln!(&mut output, "- {step}").expect("write string");
    }
    output
}

fn push_count_lines(output: &mut String, summary: &LakeFaninVerifySummary) {
    writeln!(
        output,
        "source_counts: match={} stream_required={} stream_complete={} stream_missing={} stream_quarantined={} lake_required={} lake_complete={} lake_missing={} lake_quarantined={}",
        summary.source_counts_match,
        summary.stream_required_source_count,
        summary.stream_complete_source_count,
        summary.stream_missing_source_count,
        summary.stream_quarantined_source_count,
        summary.lake_required_source_count,
        summary.lake_complete_source_count,
        summary.lake_missing_source_count,
        summary.lake_quarantined_source_count
    )
    .expect("write string");
    writeln!(
        output,
        "checksum_rollup: match={} stream={} lake={}",
        summary.checksum_rollup_match, summary.stream_checksum_rollup, summary.lake_checksum_rollup
    )
    .expect("write string");
    writeln!(
        output,
        "checks: matched={} mismatches={} warnings={} blockers={}",
        summary.matched_check_count,
        summary.mismatch_count,
        summary.warning_mismatch_count,
        summary.blocker_mismatch_count
    )
    .expect("write string");
}

fn push_mismatches(output: &mut String, summary: &LakeFaninVerifySummary) {
    writeln!(output, "mismatches:").expect("write string");
    for mismatch in &summary.mismatches {
        writeln!(
            output,
            "- {} severity={} stream={} lake={}",
            mismatch.field,
            lake_fanin_verify_mismatch_severity_label(mismatch.severity),
            mismatch.stream_value,
            mismatch.lake_value
        )
        .expect("write string");
    }
}

fn lake_fanin_verify_status_label(status: LakeFaninVerifyStatus) -> &'static str {
    match status {
        LakeFaninVerifyStatus::Match => "match",
        LakeFaninVerifyStatus::Mismatch => "mismatch",
        LakeFaninVerifyStatus::Blocked => "blocked",
    }
}

fn lake_fanin_verify_mismatch_severity_label(
    severity: LakeFaninVerifyMismatchSeverity,
) -> &'static str {
    match severity {
        LakeFaninVerifyMismatchSeverity::Warning => "warning",
        LakeFaninVerifyMismatchSeverity::Blocker => "blocker",
    }
}
