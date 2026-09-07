use std::fmt::Write as _;

use crate::{FleetControlPlaneCapabilityStatus, FleetControlPlaneSummary};

pub(crate) fn render_fleet_control_plane_text(summary: &FleetControlPlaneSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara fleet control-plane pull report").expect("write string");
    writeln!(&mut output, "verdict: {}", summary.verdict).expect("write string");
    writeln!(
        &mut output,
        "build_recommendation: {}",
        summary.build_recommendation
    )
    .expect("write string");
    writeln!(
        &mut output,
        "fleet_shape: {} flows, {} sources, {} datasets, {} tables, {} local streams, {} kafka streams, {} partitioned flows",
        summary.fleet_shape.flow_count,
        summary.fleet_shape.source_count,
        summary.fleet_shape.dataset_count,
        summary.fleet_shape.table_count,
        summary.fleet_shape.local_stream_count,
        summary.fleet_shape.kafka_stream_count,
        summary.fleet_shape.partitioned_flow_count,
    )
    .expect("write string");
    writeln!(
        &mut output,
        "topology_verdict: {}",
        summary.fleet_shape.topology_verdict
    )
    .expect("write string");
    writeln!(
        &mut output,
        "fleet_scorecard_verdict: {}",
        summary.fleet_shape.fleet_scorecard_verdict
    )
    .expect("write string");
    writeln!(
        &mut output,
        "capability_pull_count: {}",
        summary.capability_pull_count
    )
    .expect("write string");

    output.push_str("\ncapabilities:\n");
    for capability in &summary.capabilities_to_build {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            fleet_control_plane_capability_status_label(capability.status),
            capability.code,
            capability.rationale
        )
        .expect("write string");
        writeln!(&mut output, "  build_when: {}", capability.build_when).expect("write string");
    }

    if !summary.evidence_gaps.is_empty() {
        output.push_str("\nevidence_gaps:\n");
        for gap in &summary.evidence_gaps {
            writeln!(&mut output, "- {gap}").expect("write string");
        }
    }
    if !summary.identity_collisions.is_empty() {
        output.push_str("\nidentity_collisions:\n");
        for collision in &summary.identity_collisions {
            writeln!(&mut output, "- {collision}").expect("write string");
        }
    }
    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}

fn fleet_control_plane_capability_status_label(
    status: FleetControlPlaneCapabilityStatus,
) -> &'static str {
    match status {
        FleetControlPlaneCapabilityStatus::PulledNow => "pulled_now",
        FleetControlPlaneCapabilityStatus::ValidateWithPartners => "validate_with_partners",
        FleetControlPlaneCapabilityStatus::Defer => "defer",
    }
}
