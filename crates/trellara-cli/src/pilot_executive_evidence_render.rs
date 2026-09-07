use std::fmt::Write as _;

use crate::{PilotExecutiveEvidenceSummary, PilotGuideOutputFormat, Result};

pub(crate) fn render_pilot_evidence_summary(
    summary: &PilotExecutiveEvidenceSummary,
    format: PilotGuideOutputFormat,
) -> Result<String> {
    match format {
        PilotGuideOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        PilotGuideOutputFormat::Text => Ok(render_pilot_executive_evidence(summary)),
    }
}

pub(crate) fn render_pilot_executive_evidence(summary: &PilotExecutiveEvidenceSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "# Trellara Executive Evidence Brief").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "source: {}", summary.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", summary.dataset_id).expect("write string");
    writeln!(&mut output, "stream: {}", summary.stream_kind).expect("write string");
    writeln!(&mut output, "kafka_required: {}", summary.kafka_required).expect("write string");
    push_section(&mut output, "North Star", &summary.north_star);
    push_section(&mut output, "Evaluation Thesis", &summary.evaluation_thesis);
    push_section(
        &mut output,
        "Transaction Boundary",
        &summary.transaction_boundary,
    );
    push_list_section(
        &mut output,
        "Differentiated Controls",
        &summary.differentiated_controls,
        false,
    );
    push_list_section(&mut output, "Proof Commands", &summary.proof_commands, true);
    push_list_section(
        &mut output,
        "Live Evidence Required",
        &summary.live_evidence_required,
        false,
    );
    push_section(
        &mut output,
        "Executive Decision",
        &summary.executive_decision,
    );
    output
}

fn push_section(output: &mut String, title: &str, body: &str) {
    writeln!(output).expect("write string");
    writeln!(output, "## {title}").expect("write string");
    writeln!(output).expect("write string");
    writeln!(output, "{body}").expect("write string");
}

fn push_list_section(output: &mut String, title: &str, items: &[String], code: bool) {
    writeln!(output).expect("write string");
    writeln!(output, "## {title}").expect("write string");
    writeln!(output).expect("write string");
    for item in items {
        if code {
            writeln!(output, "- `{item}`").expect("write string");
        } else {
            writeln!(output, "- {item}").expect("write string");
        }
    }
}
