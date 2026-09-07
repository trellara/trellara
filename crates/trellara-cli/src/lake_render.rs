use std::fmt::Write as _;

use crate::{
    lake_completeness_state_label, lake_epoch_verification_status_label,
    lake_fanin_run_types::{LakeFaninRunStatus, LakeFaninRunSummary},
    lake_writer_plan_render::render_lake_writer_plan_text,
    LakeSparkTemplateSummary, QuickstartOutputFormat, Result,
};

pub(crate) fn render_lake_writer_plan_summary(
    plan: &trellara_lake::LakeRawCdcEpochWritePlan,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(plan)?),
        QuickstartOutputFormat::Text => Ok(render_lake_writer_plan_text(plan)),
    }
}

pub(crate) fn render_lake_fanin_run_summary(
    summary: &LakeFaninRunSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_lake_fanin_run_text(summary)),
    }
}

fn render_lake_fanin_run_text(summary: &LakeFaninRunSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara lake fan-in run").expect("write string");
    writeln!(
        &mut output,
        "dataset: {} epoch: {} mode={} status={}",
        summary.dataset_id,
        summary.epoch_id,
        summary.mode,
        lake_fanin_run_status_label(summary.trial_status)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transactions={} changes={} duplicate_replays={} skipped_dataset={} data_files={} source_buckets={} replay_safe={}",
        summary.transaction_count,
        summary.change_count,
        summary.duplicate_replay_count,
        summary.skipped_dataset_transaction_count,
        summary.data_file_count,
        summary.source_bucket_count,
        summary.replay_safe
    )
    .expect("write string");
    writeln!(
        &mut output,
        "spark_release_gate: {}",
        summary.spark_release_gate
    )
    .expect("write string");
    writeln!(
        &mut output,
        "source_ack_boundary: {}",
        summary.source_ack_boundary
    )
    .expect("write string");
    writeln!(
        &mut output,
        "catalog_backpressure_rule: {}",
        summary.catalog_backpressure_rule
    )
    .expect("write string");
    writeln!(&mut output, "bounded_trial: {}", summary.bounded_trial_note).expect("write string");
    writeln!(
        &mut output,
        "writer_plan: committers={} data_files={} state={} verification={} quarantine_rows={}",
        summary.writer_plan.committer_topology.committer_count,
        summary.writer_plan.data_file_count,
        lake_completeness_state_label(summary.writer_plan.epoch_metadata.epoch_row.state),
        lake_epoch_verification_status_label(
            summary
                .writer_plan
                .epoch_metadata
                .verification_row
                .checksum_status
        ),
        summary.writer_plan.epoch_metadata.quarantine_rows.len()
    )
    .expect("write string");
    push_fanin_quarantine_rows(&mut output, summary);
    output
}

fn push_fanin_quarantine_rows(output: &mut String, summary: &LakeFaninRunSummary) {
    if summary
        .writer_plan
        .epoch_metadata
        .quarantine_rows
        .is_empty()
    {
        return;
    }
    writeln!(output, "quarantine_evidence:").expect("write string");
    for quarantine in &summary.writer_plan.epoch_metadata.quarantine_rows {
        writeln!(
            output,
            "- source={} tx={} commit_lsn={} reason={} details={} recovery_command={}",
            quarantine.source_id.as_deref().unwrap_or("<none>"),
            quarantine.transaction_id.as_deref().unwrap_or("<none>"),
            quarantine.commit_lsn.as_deref().unwrap_or("<none>"),
            quarantine.reason,
            quarantine.details.as_deref().unwrap_or("<none>"),
            quarantine.recovery_command.as_deref().unwrap_or("<none>")
        )
        .expect("write string");
    }
}

fn lake_fanin_run_status_label(status: LakeFaninRunStatus) -> &'static str {
    match status {
        LakeFaninRunStatus::PlannedDryRun => "planned_dry_run",
        LakeFaninRunStatus::PlannedWithDuplicateReplays => "planned_with_duplicate_replays",
        LakeFaninRunStatus::BlockedNoDataFiles => "blocked_no_data_files",
        LakeFaninRunStatus::BlockedEpochNotConsumable => "blocked_epoch_not_consumable",
    }
}

pub(crate) fn render_lake_spark_template_summary(
    summary: &LakeSparkTemplateSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(summary.sql.clone()),
    }
}
