use std::fmt::Write as _;

use crate::{
    lake_completeness_state_label,
    lake_epoch_render_rows::{
        push_partition_rollup_rows, push_quarantine_entry_rows, push_source_watermark_rows,
        push_table_rollup_rows,
    },
    lake_epoch_verification_status_label, push_partition_skew, LakeEpochSummary,
    QuickstartOutputFormat, Result,
};

pub(crate) fn render_lake_epoch_summary(
    summary: &LakeEpochSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_lake_epoch_text(summary)),
    }
}

fn render_lake_epoch_text(summary: &LakeEpochSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara lake epoch").expect("write string");
    writeln!(
        &mut output,
        "dataset: {} source={} mode={} fanin_mode={}",
        summary.dataset_id, summary.source_id, summary.mode, summary.fanin_mode
    )
    .expect("write string");
    writeln!(
        &mut output,
        "epoch: {} state={} recovered_state={}",
        summary.epoch_id,
        lake_completeness_state_label(summary.state),
        summary
            .recovered_state
            .map(lake_completeness_state_label)
            .unwrap_or("none")
    )
    .expect("write string");
    writeln!(
        &mut output,
        "sources: {}/{} complete; missing={} quarantined={}",
        summary.complete_source_count,
        summary.required_source_count,
        summary.missing_source_count,
        summary.quarantined_source_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "changes: transactions={} changes={} duplicate_replays={} checksum_rollup={}",
        summary.transaction_count,
        summary.change_count,
        summary.duplicate_replay_count,
        summary.checksum_rollup
    )
    .expect("write string");
    writeln!(
        &mut output,
        "straggler_policy: {} decision={}",
        summary.straggler_policy, summary.straggler_policy_decision
    )
    .expect("write string");
    writeln!(&mut output, "manifest_digest: {}", summary.manifest_digest).expect("write string");
    writeln!(
        &mut output,
        "watermarks: global_low={} max_source={} complete={} lagging={} missing={} quarantined={} invalid_lsn={}",
        summary
            .watermark_rollup
            .global_low_watermark_lsn
            .as_deref()
            .unwrap_or("none"),
        summary
            .watermark_rollup
            .max_source_watermark_lsn
            .as_deref()
            .unwrap_or("none"),
        summary.watermark_rollup.complete_source_count,
        summary.watermark_rollup.lagging_source_count,
        summary.watermark_rollup.missing_source_count,
        summary.watermark_rollup.quarantined_source_count,
        summary.watermark_rollup.invalid_lsn_source_count
    )
    .expect("write string");
    if !summary.watermark_rollup.invalid_lsn_sources.is_empty() {
        writeln!(
            &mut output,
            "invalid_lsn_sources: {}",
            summary.watermark_rollup.invalid_lsn_sources.join(", ")
        )
        .expect("write string");
    }
    push_partition_skew(&mut output, summary);
    push_source_watermark_rows(&mut output, summary);
    push_table_rollup_rows(&mut output, summary);
    push_partition_rollup_rows(&mut output, summary);
    push_quarantine_entry_rows(&mut output, summary);
    writeln!(
        &mut output,
        "decision: {} passed={} verification={}",
        summary.customer_decision,
        summary.passed,
        lake_epoch_verification_status_label(summary.verification_status)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "visibility_boundary: {}",
        summary.visibility_boundary
    )
    .expect("write string");
    if let Some(failure) = &summary.injected_failure {
        writeln!(&mut output, "failure: {failure}").expect("write string");
    }
    writeln!(&mut output, "proof: {}", summary.proof_command).expect("write string");
    writeln!(&mut output, "next_steps:").expect("write string");
    for step in &summary.recommended_next_steps {
        writeln!(&mut output, "- {step}").expect("write string");
    }
    output
}
