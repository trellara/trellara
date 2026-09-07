use std::fmt::Write as _;

use crate::{DdlEnvelopePlanSummary, QuickstartOutputFormat, Result};

pub(crate) fn render_ddl_envelope_plan_summary(
    summary: &DdlEnvelopePlanSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_ddl_envelope_plan_text(summary)),
    }
}

fn render_ddl_envelope_plan_text(summary: &DdlEnvelopePlanSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara DDL envelope runtime plan").expect("write string");
    writeln!(
        &mut output,
        "dataset: {} source={} mode={} transaction={} commit_lsn={}",
        summary.dataset_id,
        summary.source_id,
        summary.mode,
        summary.transaction_id,
        summary.commit_lsn
    )
    .expect("write string");
    writeln!(
        &mut output,
        "source_boundary_kind={} ddl_events={} dml_changes={} executable={} global_partition_pause={}",
        summary.source_boundary_kind,
        summary.ddl_event_count,
        summary.dml_change_count,
        summary.executable,
        summary.requires_global_partition_pause
    )
    .expect("write string");
    writeln!(
        &mut output,
        "barrier_id={} barrier_schema_version={}",
        summary.barrier_id.as_deref().unwrap_or("none"),
        summary.barrier_schema_version.as_deref().unwrap_or("none")
    )
    .expect("write string");
    writeln!(
        &mut output,
        "propagation_boundary={} propagation_policy_sha256={}",
        summary.propagation_boundary, summary.propagation_policy_sha256
    )
    .expect("write string");
    push_list(
        &mut output,
        "propagation_decisions",
        &summary.propagation_decisions,
    );
    push_ddl_events(&mut output, summary);
    push_schema_versions(&mut output, summary);
    push_list(&mut output, "required_sinks", &summary.required_sinks);
    push_list(&mut output, "blockers", &summary.blockers);
    push_list(&mut output, "target_sql", &summary.target_sql);
    push_dml_replay(&mut output, summary);
    push_list(&mut output, "steps", &summary.steps);
    output
}

fn push_schema_versions(output: &mut String, summary: &DdlEnvelopePlanSummary) {
    writeln!(output, "schema_version_evidence:").expect("write string");
    if summary.schema_versions.is_empty() {
        writeln!(output, "- none").expect("write string");
        return;
    }
    for schema_version in &summary.schema_versions {
        writeln!(
            output,
            "- relation={} version={}",
            schema_version.relation, schema_version.version
        )
        .expect("write string");
    }
}

fn push_ddl_events(output: &mut String, summary: &DdlEnvelopePlanSummary) {
    writeln!(output, "ddl_event_details:").expect("write string");
    if summary.ddl_events.is_empty() {
        writeln!(output, "- none").expect("write string");
        return;
    }
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

fn push_dml_replay(output: &mut String, summary: &DdlEnvelopePlanSummary) {
    writeln!(output, "dml_replay_after_barrier:").expect("write string");
    match &summary.dml_replay_after_barrier {
        Some(replay) => writeln!(
            output,
            "- barrier_id={} release_gate={} dml_changes={} ddl_events={} ddl_events_stripped={} replay_allowed={} original_checksum={} replay_checksum={} blocked_until={}",
            replay.barrier_id.as_deref().unwrap_or("none"),
            replay.release_gate,
            replay.dml_change_count,
            replay.ddl_event_count,
            replay.ddl_events_stripped,
            replay.replay_allowed,
            replay.original_checksum,
            replay.replay_checksum,
            replay.blocked_until
        )
        .expect("write string"),
        None => writeln!(output, "- none").expect("write string"),
    }
}

fn push_list(output: &mut String, label: &str, values: &[String]) {
    writeln!(output, "{label}:").expect("write string");
    if values.is_empty() {
        writeln!(output, "- none").expect("write string");
        return;
    }
    for value in values {
        writeln!(output, "- {value}").expect("write string");
    }
}

fn empty_label(value: &str) -> &str {
    if value.is_empty() {
        "none"
    } else {
        value
    }
}
