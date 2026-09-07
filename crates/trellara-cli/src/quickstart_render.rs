use std::fmt::Write as _;

use crate::{
    render_mvp_readiness_text, render_quickstart_readiness_text, MvpReadinessSummary,
    QuickstartOutputFormat, QuickstartReadinessSummary, QuickstartSummary, Result,
};

pub(crate) fn render_quickstart_summary(
    summary: &QuickstartSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_quickstart_text(summary)),
    }
}

pub(crate) fn render_quickstart_readiness_summary(
    summary: &QuickstartReadinessSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_quickstart_readiness_text(summary)),
    }
}

pub(crate) fn render_mvp_readiness_summary(
    summary: &MvpReadinessSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_mvp_readiness_text(summary)),
    }
}

pub(crate) fn render_quickstart_text(summary: &QuickstartSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara quickstart").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "objective: {}", summary.objective).expect("write string");
    writeln!(
        &mut output,
        "time: estimated {} minutes, budget {} minutes",
        summary.estimated_minutes, summary.time_budget_minutes
    )
    .expect("write string");
    writeln!(&mut output, "evidence_bundle: {}", summary.evidence_bundle).expect("write string");
    writeln!(&mut output, "recovery: {}", summary.recovery_command).expect("write string");
    writeln!(&mut output, "commands: {}", summary.command_count).expect("write string");
    output.push_str("\nplan:\n");
    for command in &summary.commands {
        writeln!(&mut output, "{}. {}", command.step, command.command).expect("write string");
        writeln!(&mut output, "   {}", command.purpose).expect("write string");
    }
    output.push_str("\noperator_reference:\n");
    for command in &summary.operator_reference_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }
    output
}
