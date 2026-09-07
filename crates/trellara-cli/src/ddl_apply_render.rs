use std::fmt::Write as _;

use crate::{DdlApplyPlanSummary, QuickstartOutputFormat, Result};

pub(crate) fn render_ddl_apply_plan_summary(
    summary: &DdlApplyPlanSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_ddl_apply_plan_text(summary)),
    }
}

fn render_ddl_apply_plan_text(summary: &DdlApplyPlanSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara target Postgres DDL apply plan").expect("write string");
    writeln!(
        &mut output,
        "dataset: {} source={} barrier_id={}",
        summary.dataset_id, summary.source_id, summary.barrier_id
    )
    .expect("write string");
    writeln!(
        &mut output,
        "executable={} statements={} blockers={} plan_sha256={}",
        summary.executable, summary.statement_count, summary.blocker_count, summary.plan_sha256
    )
    .expect("write string");
    push_list(&mut output, "blockers", &summary.blockers);
    writeln!(&mut output, "statements:").expect("write string");
    for statement in &summary.statements {
        writeln!(
            &mut output,
            "- change={} sink={} statement_sha256={} sql={} scope={} release_gate_code={} release_gate={}",
            statement.change,
            statement.sink,
            statement.statement_sha256,
            statement.sql,
            statement.transaction_scope,
            statement.release_gate_code,
            statement.release_gate
        )
        .expect("write string");
    }
    if let Some(script) = &summary.target_postgres_transaction_script {
        writeln!(&mut output, "target_postgres_transaction_script:").expect("write string");
        writeln!(&mut output, "```sql\n{script}```").expect("write string");
    }
    push_list(
        &mut output,
        "target_postgres_ack_commands",
        &summary.target_postgres_ack_commands,
    );
    push_list(&mut output, "steps", &summary.steps);
    push_list(&mut output, "next_commands", &summary.next_commands);
    output
}

fn push_list(output: &mut String, label: &str, values: &[String]) {
    writeln!(output, "{label}:").expect("write string");
    if values.is_empty() {
        writeln!(output, "- none").expect("write string");
        return;
    }
    for value in values {
        writeln!(output, "- {value}").expect("write string");
    }
}
