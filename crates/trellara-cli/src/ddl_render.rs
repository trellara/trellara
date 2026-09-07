use std::fmt::Write as _;

use crate::{
    ddl_plan_change_kind_label, ddl_plan_compatibility_label, ddl_plan_decision_label,
    ddl_plan_verdict_label, ddl_propagation_policy_mode_label, ddl_propagation_sink_kind_label,
    DdlPlanSummary, QuickstartOutputFormat, Result,
};

pub(crate) fn render_ddl_plan_summary(
    summary: &DdlPlanSummary,
    format: QuickstartOutputFormat,
) -> Result<String> {
    match format {
        QuickstartOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        QuickstartOutputFormat::Text => Ok(render_ddl_plan_text(summary)),
    }
}
pub(crate) fn render_ddl_plan_text(summary: &DdlPlanSummary) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara schema DDL propagation plan").expect("write string");
    writeln!(
        &mut output,
        "dataset: {} source={} mode={} apply_mode={}",
        summary.dataset_id, summary.source_id, summary.mode, summary.apply_mode
    )
    .expect("write string");
    writeln!(
        &mut output,
        "verdict={} proposed={} auto_apply={} staged={} manual_review={} blocked={}",
        ddl_plan_verdict_label(summary.verdict),
        summary.proposed_change_count,
        summary.auto_apply_count,
        summary.staged_rollout_count,
        summary.manual_review_count,
        summary.blocked_count
    )
    .expect("write string");
    writeln!(
        &mut output,
        "transaction_boundary_rule: {}",
        summary.transaction_boundary_rule
    )
    .expect("write string");
    writeln!(
        &mut output,
        "propagation: barrier_id={} scope={} cdc_boundary={} row_visibility={} dml_after_barrier_held={} partition_pause={} sinks={} required_acks={} ack_quorum={}",
        summary.propagation.barrier_id,
        summary.propagation.barrier_scope,
        summary.propagation.cdc_transaction_boundary,
        summary.propagation.row_visibility_mode,
        summary.propagation.dml_after_barrier_held,
        summary.propagation.requires_global_partition_pause,
        summary.propagation.sink_count,
        summary.propagation.required_ack_count,
        summary.propagation.ack_quorum
    )
    .expect("write string");
    writeln!(&mut output, "policy_modes:").expect("write string");
    for policy in &summary.propagation.policy_modes {
        writeln!(
            &mut output,
            "- {} active={} changes={} release_rule={} approval_evidence={}",
            ddl_propagation_policy_mode_label(policy.mode),
            policy.active,
            policy.change_count,
            policy.release_rule,
            policy.approval_evidence
        )
        .expect("write string");
    }
    writeln!(&mut output, "sinks:").expect("write string");
    for sink in &summary.propagation.sinks {
        writeln!(
            &mut output,
            "- {} kind={} ack={} evidence={}",
            sink.name,
            ddl_propagation_sink_kind_label(sink.kind),
            sink.required_ack,
            sink.ack_evidence
        )
        .expect("write string");
    }
    writeln!(&mut output, "phases:").expect("write string");
    for phase in &summary.propagation.phases {
        writeln!(
            &mut output,
            "- {} {}: {}",
            phase.order, phase.name, phase.release_condition
        )
        .expect("write string");
    }
    writeln!(&mut output, "release_gates:").expect("write string");
    for gate in &summary.propagation.release_gates {
        writeln!(
            &mut output,
            "- {} evidence={} opens_when={}",
            gate.name, gate.required_evidence, gate.opens_when
        )
        .expect("write string");
    }
    if !summary.blockers.is_empty() {
        writeln!(&mut output, "blockers:").expect("write string");
        for blocker in &summary.blockers {
            writeln!(
                &mut output,
                "- {} relation={} compatibility={} impact={}",
                blocker.change,
                blocker.relation.as_deref().unwrap_or("unknown"),
                ddl_plan_compatibility_label(blocker.compatibility),
                blocker.release_impact
            )
            .expect("write string");
            writeln!(&mut output, "  reason: {}", blocker.reason).expect("write string");
        }
    }
    writeln!(&mut output, "changes:").expect("write string");
    for change in &summary.changes {
        writeln!(
            &mut output,
            "- {} object={} relation={} compatibility={} decision={}",
            ddl_plan_change_kind_label(change.kind),
            change.object,
            change.relation.as_deref().unwrap_or("unknown"),
            ddl_plan_compatibility_label(change.compatibility),
            ddl_plan_decision_label(change.decision)
        )
        .expect("write string");
        writeln!(&mut output, "  reason: {}", change.reason).expect("write string");
        writeln!(&mut output, "  boundary: {}", change.boundary_rule).expect("write string");
        if let Some(sql) = &change.target_postgres_sql {
            writeln!(&mut output, "  target_postgres_sql: {sql}").expect("write string");
        }
        for action in &change.propagation_actions {
            writeln!(
                &mut output,
                "  action[{}]: {} ack={}",
                action.sink, action.action, action.ack_condition
            )
            .expect("write string");
        }
    }
    writeln!(&mut output, "next_commands:").expect("write string");
    for command in &summary.next_commands {
        writeln!(&mut output, "- {command}").expect("write string");
    }
    output
}
