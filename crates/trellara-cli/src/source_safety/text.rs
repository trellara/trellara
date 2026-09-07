use std::fmt::Write as _;

use trellara_pg_capture::{ReplicationSlotStatus, SubscriptionConflictStats};

use crate::{
    flow_alert_severity_label, flow_health_status_label, source_safety_grade_label,
    source_slot_summary_line, FlowHealthStatus, SourceSafetyFactor, SourceSafetyGrade,
    SourceSafetyInitRecommendation,
};

pub(crate) struct SourceSafetyTextInput<'a> {
    pub(crate) source_id: &'a str,
    pub(crate) dataset_id: &'a str,
    pub(crate) mode: &'a str,
    pub(crate) read_only: Option<bool>,
    pub(crate) score: u8,
    pub(crate) grade: SourceSafetyGrade,
    pub(crate) status: FlowHealthStatus,
    pub(crate) table_count: Option<usize>,
    pub(crate) unsafe_table_count: Option<usize>,
    pub(crate) slot: Option<&'a ReplicationSlotStatus>,
    pub(crate) subscription_conflicts: &'a [SubscriptionConflictStats],
    pub(crate) factors: &'a [SourceSafetyFactor],
    pub(crate) recommended_actions: &'a [String],
    pub(crate) init_recommendation: Option<&'a SourceSafetyInitRecommendation>,
    pub(crate) init_config_written: Option<&'a str>,
}

pub(crate) fn render_source_safety_text(input: SourceSafetyTextInput<'_>) -> String {
    let mut output = String::new();
    writeln!(&mut output, "Trellara source safety").expect("write string");
    writeln!(&mut output, "source: {}", input.source_id).expect("write string");
    writeln!(&mut output, "dataset: {}", input.dataset_id).expect("write string");
    writeln!(&mut output, "mode: {}", input.mode).expect("write string");
    writeln!(
        &mut output,
        "status: {}",
        flow_health_status_label(input.status)
    )
    .expect("write string");
    writeln!(
        &mut output,
        "grade: {} ({}/100)",
        source_safety_grade_label(input.grade),
        input.score
    )
    .expect("write string");

    if let Some(read_only) = input.read_only {
        writeln!(&mut output, "read_only: {read_only}").expect("write string");
    }
    if let (Some(table_count), Some(unsafe_table_count)) =
        (input.table_count, input.unsafe_table_count)
    {
        writeln!(
            &mut output,
            "tables: {table_count} checked, {unsafe_table_count} unsafe"
        )
        .expect("write string");
    }
    if let Some(slot) = input.slot {
        writeln!(&mut output, "{}", source_slot_summary_line(slot)).expect("write string");
    }
    if !input.subscription_conflicts.is_empty() {
        output.push_str("\nsubscription_conflicts:\n");
        for stat in input.subscription_conflicts {
            writeln!(
                &mut output,
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

    output.push_str("\nfindings:\n");
    if input.factors.is_empty() {
        output.push_str("- none\n");
    } else {
        for factor in input.factors {
            writeln!(
                &mut output,
                "- [{}] {}: {}",
                flow_alert_severity_label(factor.severity),
                factor.code,
                factor.evidence
            )
            .expect("write string");
        }
    }

    output.push_str("\nrecommended_actions:\n");
    if input.recommended_actions.is_empty() {
        output.push_str("- none\n");
    } else {
        for action in input.recommended_actions {
            writeln!(&mut output, "- {action}").expect("write string");
        }
    }
    if let Some(recommendation) = input.init_recommendation {
        output.push_str("\ninit_recommendation:\n");
        writeln!(&mut output, "- command: {}", recommendation.command).expect("write string");
        writeln!(&mut output, "  output: {}", recommendation.output).expect("write string");
        writeln!(
            &mut output,
            "  tables: {} primary_key={} evaluation_ready={}",
            recommendation.table_count, recommendation.primary_key, recommendation.evaluation_ready
        )
        .expect("write string");
        writeln!(&mut output, "  note: {}", recommendation.note).expect("write string");
    }
    if let Some(path) = input.init_config_written {
        writeln!(&mut output, "\ninit_config_written: {path}").expect("write string");
    }

    output
}
