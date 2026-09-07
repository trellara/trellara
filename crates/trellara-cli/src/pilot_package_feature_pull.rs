use std::fmt::Write as _;
use std::path::Path;

use crate::{pilot_scorecard_status_label, PilotScorecardSummary, TrellaraConfig};

pub(crate) fn render_pilot_feature_pull_list(
    config: &TrellaraConfig,
    config_path: &Path,
    scorecard: &PilotScorecardSummary,
) -> String {
    let config_display = config_path.display();
    let mut output = String::new();
    writeln!(&mut output, "# Design-Partner Feature Pull List").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "config: {config_display}").expect("write string");
    writeln!(&mut output, "source: {}", config.source.id).expect("write string");
    writeln!(&mut output, "dataset: {}", config.dataset.id).expect("write string");
    writeln!(&mut output, "mode: {}", config.status_mode()).expect("write string");
    writeln!(&mut output, "verdict: {}", scorecard.verdict).expect("write string");
    writeln!(&mut output).expect("write string");

    output.push_str("## Validate Before Building Cloud\n\n");
    output.push_str("- Hosted fleet topology: only if partners need to manage many configs, roles, and target groups centrally.\n");
    output.push_str("- Policy and audit workflow: only if source-safety decisions require approvals, exceptions, or compliance evidence.\n");
    output.push_str("- Hosted correctness reports: only if partners want immutable, shareable evidence tied to commits and runtime flow IDs.\n");
    output.push_str("- Deployment orchestration: only if recurring rollouts across many Postgres nodes create measurable operational burden.\n");
    output.push_str("- Hosted stream or BYOC stream management: only if local transport or existing Kafka ownership becomes a bottleneck.\n\n");

    output.push_str("## Evidence To Collect\n\n");
    output.push_str("- Number of Postgres databases the partner would put under verified replication after the first flow passes.\n");
    output.push_str("- Source-safety issues discovered before CDC starts, especially WAL retention and failover-slot readiness.\n");
    output.push_str("- Recovery actions used during the failure drill and whether they were sufficient without platform-team intervention.\n");
    output.push_str("- Security or procurement evidence missing from `executive-evidence.md`, `proof-bundle.md`, or the correctness report.\n\n");

    output.push_str("## Current Gate Status\n\n");
    for gate in &scorecard.gates {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            pilot_scorecard_status_label(gate.status),
            gate.code,
            gate.proof_command
        )
        .expect("write string");
    }
    output
}
