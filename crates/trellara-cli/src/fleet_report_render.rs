use std::fmt::Write as _;

use crate::{push_fleet_report_flow_text, FleetReportSummary};

pub(crate) fn render_fleet_report_text(summary: &FleetReportSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara fleet report").expect("write string");
    writeln!(&mut output, "verdict: {}", summary.topology_verdict).expect("write string");
    writeln!(&mut output, "flows: {}", summary.flow_count).expect("write string");
    writeln!(&mut output, "sources: {}", summary.source_count).expect("write string");
    writeln!(&mut output, "datasets: {}", summary.dataset_count).expect("write string");
    writeln!(&mut output, "tables: {}", summary.table_count).expect("write string");
    writeln!(
        &mut output,
        "targets_configured: {}/{}",
        summary.target_configured_count, summary.flow_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "streams: local={} kafka={}",
        summary.local_stream_count, summary.kafka_stream_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "modes: partitioned={} strict_chunked={}",
        summary.partitioned_flow_count, summary.strict_chunked_flow_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "convergence_gates: {} total, {} blocked_by_config",
        summary.convergence_gate_count, summary.blocked_convergence_gate_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "recovery_drills: {}",
        summary.recovery_drill_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "lake_fanin: verdict={} ready={} publishable_with_gaps={} blocked={}",
        summary.lake_fanin_verdict,
        summary.lake_ready_flow_count,
        summary.lake_publishable_with_gaps_flow_count,
        summary.lake_blocked_flow_count
    )
    .expect("write string");

    output.push_str("\nflows:\n");
    for flow in &summary.flows {
        push_fleet_report_flow_text(&mut output, flow);
    }

    if !summary.warnings.is_empty() {
        output.push_str("\nwarnings:\n");
        for warning in &summary.warnings {
            writeln!(&mut output, "- {warning}").expect("write string");
        }
    }

    output.push_str("\nproof_commands:\n");
    for command in &summary.proof_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}
