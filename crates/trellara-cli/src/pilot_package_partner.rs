use std::fmt::Write as _;
use std::path::Path;

use crate::{DatasetMode, StreamConfig, TrellaraConfig};

pub(crate) fn render_pilot_deployment_guide(config: &TrellaraConfig, config_path: &Path) -> String {
    let config_display = config_path.display();
    let mut output = String::new();
    writeln!(&mut output, "# Trellara Design-Partner Deployment Guide").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "config: {config_display}").expect("write string");
    writeln!(&mut output, "source: {}", config.source.id).expect("write string");
    writeln!(&mut output, "dataset: {}", config.dataset.id).expect("write string");
    writeln!(&mut output, "mode: {}", config.status_mode()).expect("write string");
    writeln!(&mut output).expect("write string");

    output.push_str("## Pilot Objective\n\n");
    output.push_str("Prove one PostgreSQL source-to-target replication flow with source safety, snapshot handoff, transaction-boundary evidence, convergence verification, and a recovery drill before broad fleet rollout.\n\n");

    output.push_str("## Environment Checklist\n\n");
    output.push_str("- Source PostgreSQL allows logical replication and exposes the selected publication and slot posture.\n");
    output.push_str("- Target PostgreSQL has matching required columns plus any configured target-owned columns.\n");
    output.push_str("- Operators can run `source-safety`, `preflight`, `contract-test`, `snapshot`, `run`, `verify`, `status`, and `pilot-package` from the same config.\n");
    if matches!(config.stream, StreamConfig::Local { .. }) {
        output.push_str(
            "- Brokerless local stream storage is on durable disk and uses the configured fsync policy.\n",
        );
    } else {
        output.push_str(
            "- Kafka or Redpanda topics exist for the selected transaction-boundary mode.\n",
        );
    }
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        output.push_str("- Partitioned flows identify the ownership key and agree on null-key and key-change policy before launch.\n");
    }
    writeln!(&mut output).expect("write string");

    output.push_str("## Deployment Steps\n\n");
    writeln!(
        &mut output,
        "1. Source safety: `trellara check --config {config_display} --format text`"
    )
    .expect("write string");
    writeln!(
        &mut output,
        "2. Contract preflight: `trellara preflight --config {config_display}` and `trellara contract-test --config {config_display}`"
    )
    .expect("write string");
    writeln!(
        &mut output,
        "3. Snapshot handoff: `trellara snapshot --config {config_display} --run-id pilot-snapshot-1`"
    )
    .expect("write string");
    if matches!(config.stream, StreamConfig::Local { .. }) {
        writeln!(
            &mut output,
            "4. Verified local loop: `trellara run --local --verify --format text --config {config_display} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100`"
        )
        .expect("write string");
    } else {
        writeln!(
            &mut output,
            "4. Runtime services: `trellara relay --config {config_display}`, `trellara apply --config {config_display}`, then `trellara verify --config {config_display}`"
        )
        .expect("write string");
    }
    writeln!(
        &mut output,
        "5. Operator proof: `trellara status --config {config_display} --view report --format text`"
    )
    .expect("write string");
    writeln!(
        &mut output,
        "6. Support bundle: `trellara status --config {config_display} --view diagnostics --format text`"
    )
    .expect("write string");
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        writeln!(
            &mut output,
            "7. Global visibility gate: `trellara partition-watermarks --config {config_display}`"
        )
        .expect("write string");
    }
    writeln!(&mut output).expect("write string");

    output.push_str("## Go/No-Go Rule\n\n");
    output.push_str("Advance only when source-safety has no critical blockers, snapshot handoff is complete, target verification converges, and any failure appears as an explicit recovery action instead of silent checkpoint advancement.\n");
    output
}
