use std::fmt::Write as _;

use crate::{
    labels::{grade_label, severity_label, status_label},
    slot_evidence::slot_summary_line,
    CheckSummary,
};

pub(crate) fn render_check_text(summary: &CheckSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara check").expect("write string");
    writeln!(&mut output, "database: {}", summary.database).expect("write string");
    writeln!(&mut output, "read_only: {}", summary.read_only).expect("write string");
    writeln!(
        &mut output,
        "managed_postgres_ready: {}",
        summary.managed_postgres_ready
    )
    .expect("write string");
    writeln!(&mut output, "status: {}", status_label(summary.status)).expect("write string");
    writeln!(
        &mut output,
        "grade: {} ({}/100)",
        grade_label(summary.grade),
        summary.score
    )
    .expect("write string");
    writeln!(
        &mut output,
        "tables: {} checked, {} unsafe",
        summary.table_count, summary.unsafe_table_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "logical_slots: {} checked, {} at_risk",
        summary.logical_slot_count, summary.at_risk_slot_count
    )
    .expect("write string");

    push_slots(&mut output, summary);
    push_subscription_conflicts(&mut output, summary);
    push_findings(&mut output, summary);
    push_actions(&mut output, summary);
    output.push_str(
        "\nexact_read_only_queries: run with --format html to include the SQL appendix\n",
    );
    output
}

fn push_slots(output: &mut String, summary: &CheckSummary) {
    if summary.logical_slots.is_empty() {
        output.push_str("\nlogical_slot_evidence:\n- none\n");
        return;
    }
    output.push_str("\nlogical_slot_evidence:\n");
    for slot in &summary.logical_slots {
        writeln!(output, "- {}", slot_summary_line(slot)).expect("write string");
    }
}

fn push_subscription_conflicts(output: &mut String, summary: &CheckSummary) {
    if summary.subscription_conflicts.is_empty() {
        return;
    }
    output.push_str("\nsubscription_conflicts:\n");
    for stat in &summary.subscription_conflicts {
        writeln!(
            output,
            "- {} apply_errors={} sync_errors={} conflicts={} update_missing={} delete_missing={}",
            stat.subscription_name,
            stat.apply_error_count,
            stat.sync_error_count,
            stat.conflicts.total(),
            stat.conflicts.update_missing,
            stat.conflicts.delete_missing
        )
        .expect("write string");
    }
}

fn push_findings(output: &mut String, summary: &CheckSummary) {
    output.push_str("\nfindings:\n");
    if summary.findings.is_empty() {
        output.push_str("- none\n");
        return;
    }
    for factor in &summary.findings {
        writeln!(
            output,
            "- [{}] {}: {}",
            severity_label(factor.severity),
            factor.code,
            factor.evidence
        )
        .expect("write string");
    }
}

fn push_actions(output: &mut String, summary: &CheckSummary) {
    output.push_str("\nrecommended_actions:\n");
    if summary.recommended_actions.is_empty() {
        output.push_str("- none\n");
        return;
    }
    for action in &summary.recommended_actions {
        writeln!(output, "- {action}").expect("write string");
    }
}
