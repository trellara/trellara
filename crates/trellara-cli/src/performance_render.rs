use std::fmt::Write as _;

use crate::{PerformanceEnvelopeSummary, PilotGuideOutputFormat, Result};

pub(crate) fn render_performance_envelope_summary(
    summary: &PerformanceEnvelopeSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_performance_envelope_text(summary)),
    }
}

fn render_performance_envelope_text(summary: &PerformanceEnvelopeSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara performance envelope").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "mode: {}", summary.mode).expect("write string");
    writeln!(&mut output, "stream: {}", summary.stream_kind).expect("write string");
    writeln!(
        &mut output,
        "quickstart_target: {} minute estimate within {} minute budget",
        summary.quickstart_estimated_minutes, summary.quickstart_time_budget_minutes
    )
    .expect("write string");
    writeln!(
        &mut output,
        "default_local_loop_bound: {} transactions / {} messages",
        summary.default_relay_max_transactions, summary.default_apply_max_messages
    )
    .expect("write string");
    writeln!(
        &mut output,
        "source_capture_contract: {}",
        summary.source_capture_contract
    )
    .expect("write string");
    writeln!(
        &mut output,
        "stream_spill_threshold_changes: {}",
        summary.stream_spill_threshold_changes
    )
    .expect("write string");
    writeln!(
        &mut output,
        "stream_spill_threshold_max_changes: {}",
        summary.stream_spill_threshold_max_changes
    )
    .expect("write string");
    writeln!(
        &mut output,
        "stream_spill_location: {}",
        summary.stream_spill_location
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transaction_boundary_cost: {}",
        summary.transaction_boundary_cost
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transport_durability_cost: {}",
        summary.transport_durability_cost
    )
    .expect("write string");
    writeln!(
        &mut output,
        "measurement_note: {}",
        summary.measurement_note
    )
    .expect("write string");

    output.push_str("\nexpected_bottlenecks:\n");
    for item in &summary.expected_bottlenecks {
        writeln!(&mut output, "- {}: {}", item.code, item.summary).expect("write string");
        writeln!(&mut output, "  evidence: {}", item.evidence).expect("write string");
    }

    output.push_str("\ntuning_levers:\n");
    for item in &summary.tuning_levers {
        writeln!(&mut output, "- {}: {}", item.code, item.summary).expect("write string");
        writeln!(&mut output, "  evidence: {}", item.evidence).expect("write string");
    }

    output.push_str("\nproof_commands:\n");
    for command in &summary.proof_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}
