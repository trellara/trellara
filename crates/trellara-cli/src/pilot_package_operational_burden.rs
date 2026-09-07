use std::fmt::Write as _;
use std::path::Path;

use crate::{DatasetMode, StreamConfig, TrellaraConfig};

pub(crate) fn render_pilot_operational_burden_notes(
    config: &TrellaraConfig,
    config_path: &Path,
) -> String {
    let config_display = config_path.display();
    let mut output = String::new();
    writeln!(&mut output, "# Operational Burden Notes").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "config: {config_display}").expect("write string");
    writeln!(&mut output, "mode: {}", config.status_mode()).expect("write string");
    writeln!(&mut output).expect("write string");

    output.push_str("## Before Trellara\n\n");
    output.push_str("- CDC source risk is often inferred from slot lag, WAL growth, and destination symptoms after the source is already under pressure.\n");
    output.push_str("- Transaction-boundary handling usually lives in consumer code or separate metadata topics, making recovery reviews slow.\n");
    output.push_str("- Resync decisions depend on operator judgment when target drift, schema changes, or missing WAL appears.\n");
    output.push_str("- Design reviews must collect source, stream, target, validation, and incident evidence from different tools.\n\n");

    output.push_str("## With This Pilot\n\n");
    output.push_str("- `source-safety` turns slot, WAL, failover-slot, replica identity, and table-readiness evidence into a preflight gate.\n");
    output.push_str("- `status --view report --format text` gives one operator-readable proof checklist for checkpoint durability, transaction boundaries, quarantine, and checksum status.\n");
    output.push_str("- `status --view diagnostics --format text` packages report, alerts, metrics, latest failure, and attachment commands for escalation.\n");
    output.push_str("- `repair-plan` and quarantine commands turn recovery into named, replay-aware actions instead of manual offset guessing.\n");
    if matches!(config.stream, StreamConfig::Local { .. }) {
        output.push_str("- Brokerless local transport removes the operational burden of introducing Kafka for the first evaluation.\n");
    }
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        output.push_str(
            "- Partition watermarks define the safe global visibility point for partitioned scale mode.\n",
        );
    }
    writeln!(&mut output).expect("write string");

    output.push_str("## Incident Notes To Capture During Pilot\n\n");
    output.push_str("- Minutes from first alert to named recovery action.\n");
    output.push_str("- Whether operators could identify the source checkpoint, target checkpoint, and transaction boundary from one report.\n");
    output.push_str("- Whether a replay or reseed decision required manual stream inspection beyond Trellara's recommended actions.\n");
    output.push_str("- Any evidence needed by security, platform, or database teams that is missing from this package.\n");
    output
}
