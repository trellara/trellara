use std::fmt::Write as _;
use std::path::Path;

use crate::{
    pilot_scorecard_status_label, DatasetMode, PilotGuideSummary, PilotScorecardSummary,
    StreamConfig, TrellaraConfig,
};

pub(crate) fn render_pilot_proof_bundle(
    config: &TrellaraConfig,
    config_path: &Path,
    guide: &PilotGuideSummary,
    scorecard: &PilotScorecardSummary,
) -> String {
    let config_display = config_path.display();
    let mut output = String::new();
    writeln!(&mut output, "# Trellara Proof Bundle").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "config: {config_display}").expect("write string");
    writeln!(&mut output, "source: {}", config.source.id).expect("write string");
    writeln!(&mut output, "dataset: {}", config.dataset.id).expect("write string");
    writeln!(&mut output, "mode: {}", config.status_mode()).expect("write string");
    writeln!(&mut output, "stream: {}", guide.stream_kind).expect("write string");
    writeln!(&mut output, "kafka_required: {}", guide.kafka_required).expect("write string");
    writeln!(&mut output).expect("write string");

    output.push_str("## Decision Rule\n\n");
    output.push_str("Approve the pilot only when source safety has no critical blockers, the transaction boundary is verified, convergence evidence is current, and any failure is represented as a named quarantine or repair action instead of silent checkpoint advancement.\n\n");

    output.push_str("## Proof Chain\n\n");
    writeln!(
        &mut output,
        "0. Enterprise evaluation: `trellara evaluate --config {config_display} --format text`"
    )
    .expect("write string");
    output.push_str("   Evidence: buyer-facing readiness verdict, recommended mode, transaction-boundary contract, differentiators, live evidence required, and blockers.\n");
    writeln!(
        &mut output,
        "1. Source readiness: `trellara check --config {config_display} --format text`"
    )
    .expect("write string");
    output.push_str("   Evidence: slot posture, WAL retention, failover-slot readiness, replica identity, table selection, and downstream subscription conflicts.\n");
    writeln!(
        &mut output,
        "2. Contract preflight: `trellara contract-test --config {config_display}`"
    )
    .expect("write string");
    output.push_str("   Evidence: schema fingerprints, target compatibility, partition-key policy, and transaction-boundary contract checks.\n");
    writeln!(
        &mut output,
        "3. Snapshot handoff: `trellara snapshot --config {config_display} --run-id pilot-snapshot-1`"
    )
    .expect("write string");
    output.push_str("   Evidence: copied table progress, exported consistency point, durable handoff watermark, snapshot_handoff_blocker_codes, and snapshot_handoff_recovery_actions before stream replay.\n");
    writeln!(
        &mut output,
        "4. Runtime proof: `trellara status --config {config_display} --view report --format text`"
    )
    .expect("write string");
    output.push_str("   Evidence: source and target checkpoints, transaction-boundary status, runtime barrier_pending_blockers, checksum status, partition watermarks, and recovery recommendations.\n");
    writeln!(
        &mut output,
        "5. Support bundle: `trellara status --config {config_display} --view diagnostics --format text`"
    )
    .expect("write string");
    output.push_str("   Evidence: report, alerts, repair plan, metrics, latest failure, and attachment commands for design-partner review.\n");
    if config.target.is_some() {
        writeln!(
            &mut output,
            "6. Convergence: `trellara verify --config {config_display}`"
        )
        .expect("write string");
        output.push_str("   Evidence: source and target row counts, checksums, table scope, and checkpoint catch-up.\n");
    }
    if matches!(config.stream, StreamConfig::Local { .. }) {
        writeln!(
            &mut output,
            "7. Brokerless stream inspection: `trellara stream inspect-local --config {config_display}`"
        )
        .expect("write string");
        output.push_str("   Evidence: topic depth, pending messages, cursor health, barrier topics, and local durability posture.\n");
    }
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        writeln!(
            &mut output,
            "8. Partitioned visibility: `trellara partition-watermarks --config {config_display}`"
        )
        .expect("write string");
        output.push_str("   Evidence: every partition has checkpoint evidence before global current-state visibility advances.\n");
    }
    output.push_str(
        "9. Transaction inspection: `trellara inspect-transaction --file <envelope.pb> --format text`\n",
    );
    output.push_str("   Evidence: checksum status, affected tables, manifest `boundary_mode`, strict chunk or partition manifest event-count checks, participating chunks or partitions, and the commit-marker manifest checksum.\n\n");

    output.push_str("## Boundary Contract\n\n");
    writeln!(&mut output, "{}", guide.transaction_boundary).expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(
        &mut output,
        "large_transaction_boundary: {}",
        guide.capture_spill_boundary
    )
    .expect("write string");
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        output.push_str("partitioned_scale_mode: global visibility requires the manifest, commit marker, and complete partition watermarks; partition-local consumers remain intentionally incomplete for multi-partition transactions.\n");
    }
    if config.dataset.strict_chunking.is_some() {
        output.push_str("strict_chunking: oversized transactions are visible only after every chunk, the manifest, and the commit marker are durable.\n");
    }
    writeln!(&mut output).expect("write string");

    output.push_str("## Acceptance Gates\n\n");
    for gate in &scorecard.gates {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            pilot_scorecard_status_label(gate.status),
            gate.code,
            gate.acceptance
        )
        .expect("write string");
    }
    writeln!(&mut output).expect("write string");

    output.push_str("## Differentiation Evidence\n\n");
    output.push_str("- Correctness is exposed as a product surface through source-safety, status reports, diagnostics, proof bundles, and the static correctness report.\n");
    output.push_str("- CDC transaction boundaries are inspectable at the envelope level, not inferred from downstream row counts.\n");
    output.push_str("- Partitioned scale mode offers throughput flexibility while preserving a conservative global visibility gate.\n");
    output.push_str("- Recovery is fail-closed: target divergence becomes quarantine plus replay-ready workflow, not hidden lag or silent skips.\n");
    if matches!(config.stream, StreamConfig::Local { .. }) {
        output.push_str("- Brokerless evaluation removes Kafka adoption as a prerequisite for the first verified pilot.\n");
    }
    output.push_str("\n## Package Integrity\n\n");
    output.push_str("Use `manifest.json` to verify byte counts and SHA-256 digests for every exported artifact in this package.\n");
    output
}
