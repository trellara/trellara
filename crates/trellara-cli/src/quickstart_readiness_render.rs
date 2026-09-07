use std::fmt::Write as _;

use crate::{MvpReadinessSummary, QuickstartReadinessSummary};

pub(crate) fn render_quickstart_readiness_text(summary: &QuickstartReadinessSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara quickstart readiness").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "ready: {}", summary.ready).expect("write string");
    if let Some(estimated_minutes) = summary.estimated_minutes {
        writeln!(
            &mut output,
            "time: estimated {} minutes, budget {} minutes",
            estimated_minutes, summary.time_budget_minutes
        )
        .expect("write string");
    } else {
        writeln!(
            &mut output,
            "time: budget {} minutes after readiness passes",
            summary.time_budget_minutes
        )
        .expect("write string");
    }
    writeln!(
        &mut output,
        "checks: {}/{} passed",
        summary.passed_check_count, summary.check_count
    )
    .expect("write string");
    if let Some(evidence_bundle) = &summary.evidence_bundle {
        writeln!(&mut output, "evidence_bundle: {evidence_bundle}").expect("write string");
    }
    if let Some(recovery_command) = &summary.recovery_command {
        writeln!(&mut output, "recovery: {recovery_command}").expect("write string");
    }

    output.push_str("\nchecks:\n");
    for check in &summary.checks {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            quickstart_check_status_label(check.passed),
            check.code,
            check.message
        )
        .expect("write string");
        if let Some(fix) = &check.fix {
            writeln!(&mut output, "  fix: {fix}").expect("write string");
        }
    }

    let section_name = if summary.ready {
        "next_commands"
    } else {
        "fix_commands"
    };
    writeln!(&mut output, "\n{section_name}:").expect("write string");
    if summary.next_commands.is_empty() {
        output.push_str("- none\n");
    } else {
        for command in &summary.next_commands {
            writeln!(&mut output, "- {command}").expect("write string");
        }
    }

    output
}

pub(crate) fn render_mvp_readiness_text(summary: &MvpReadinessSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara MVP readiness").expect("write string");
    writeln!(&mut output, "config: {}", summary.config).expect("write string");
    writeln!(&mut output, "ready: {}", summary.ready).expect("write string");
    writeln!(
        &mut output,
        "criteria: {}/{} passed",
        summary.passed_criterion_count, summary.criterion_count
    )
    .expect("write string");

    output.push_str("\ncriteria:\n");
    for criterion in &summary.criteria {
        writeln!(
            &mut output,
            "- [{}] {}: {}",
            quickstart_check_status_label(criterion.passed),
            criterion.code,
            criterion.evidence
        )
        .expect("write string");
        writeln!(&mut output, "  proof: {}", criterion.proof_command).expect("write string");
    }

    output.push_str("\npriority_next_commands:\n");
    for command in &summary.priority_next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output.push_str("\nnext_commands:\n");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }

    output
}

fn quickstart_check_status_label(passed: bool) -> &'static str {
    if passed {
        "pass"
    } else {
        "fail"
    }
}
