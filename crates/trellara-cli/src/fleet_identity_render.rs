use std::fmt::Write as _;

use crate::{FleetIdentityAuditSummary, FleetIdentityStatus};

pub(crate) fn render_fleet_identity_audit_text(summary: &FleetIdentityAuditSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara fleet identity audit").expect("write string");
    writeln!(&mut output, "verdict: {}", summary.verdict).expect("write string");
    writeln!(
        &mut output,
        "flows: {} total, {} unique, {} duplicate",
        summary.flow_count, summary.unique_flow_count, summary.duplicate_flow_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "sources: {} datasets: {}",
        summary.source_count, summary.dataset_count
    )
    .expect("write string");

    output.push_str("\nflows:\n");
    for flow in &summary.flows {
        writeln!(
            &mut output,
            "- [{}] {} config={} source={} dataset={} mode={} stream={}",
            fleet_identity_status_label(flow.status),
            flow.flow_id,
            flow.config,
            flow.source_id,
            flow.dataset_id,
            flow.mode,
            flow.stream_kind
        )
        .expect("write string");
    }

    if !summary.duplicate_groups.is_empty() {
        output.push_str("\nduplicate_groups:\n");
        for group in &summary.duplicate_groups {
            writeln!(
                &mut output,
                "- {}: {}",
                group.flow_id,
                group.configs.join(", ")
            )
            .expect("write string");
            writeln!(&mut output, "  remediation: {}", group.remediation).expect("write string");
        }
    }

    output.push_str("\nremediation:\n");
    for step in &summary.remediation {
        writeln!(&mut output, "- {step}").expect("write string");
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

fn fleet_identity_status_label(status: FleetIdentityStatus) -> &'static str {
    match status {
        FleetIdentityStatus::Unique => "unique",
        FleetIdentityStatus::Duplicate => "duplicate",
    }
}
