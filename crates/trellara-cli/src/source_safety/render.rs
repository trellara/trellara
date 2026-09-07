use crate::{
    render_source_safety_html, render_source_safety_text, DirectSourceSafetySummary, Result,
    SourceSafetyOutputFormat, SourceSafetySummary, SourceSafetyTextInput,
};

pub(crate) fn render_source_safety_summary(
    summary: &SourceSafetySummary,
    format: SourceSafetyOutputFormat,
) -> Result<String> {
    match format {
        SourceSafetyOutputFormat::Html => Ok(render_source_safety_html(SourceSafetyTextInput {
            source_id: &summary.source_id,
            dataset_id: &summary.dataset_id,
            mode: &summary.mode,
            read_only: None,
            score: summary.score,
            grade: summary.grade,
            status: summary.status,
            table_count: None,
            unsafe_table_count: None,
            slot: Some(&summary.slot),
            subscription_conflicts: &summary.subscription_conflicts,
            factors: &summary.factors,
            recommended_actions: &summary.recommended_actions,
            init_recommendation: None,
            init_config_written: None,
        })),
        SourceSafetyOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        SourceSafetyOutputFormat::Text => Ok(render_source_safety_text(SourceSafetyTextInput {
            source_id: &summary.source_id,
            dataset_id: &summary.dataset_id,
            mode: &summary.mode,
            read_only: None,
            score: summary.score,
            grade: summary.grade,
            status: summary.status,
            table_count: None,
            unsafe_table_count: None,
            slot: Some(&summary.slot),
            subscription_conflicts: &summary.subscription_conflicts,
            factors: &summary.factors,
            recommended_actions: &summary.recommended_actions,
            init_recommendation: None,
            init_config_written: None,
        })),
    }
}

pub(crate) fn render_direct_source_safety_summary(
    summary: &DirectSourceSafetySummary,
    format: SourceSafetyOutputFormat,
) -> Result<String> {
    match format {
        SourceSafetyOutputFormat::Html => Ok(render_source_safety_html(SourceSafetyTextInput {
            source_id: &summary.source_id,
            dataset_id: &summary.dataset_id,
            mode: &summary.mode,
            read_only: Some(summary.read_only),
            score: summary.score,
            grade: summary.grade,
            status: summary.status,
            table_count: Some(summary.table_count),
            unsafe_table_count: Some(summary.unsafe_table_count),
            slot: Some(&summary.slot),
            subscription_conflicts: &summary.subscription_conflicts,
            factors: &summary.factors,
            recommended_actions: &summary.recommended_actions,
            init_recommendation: summary.init_recommendation.as_ref(),
            init_config_written: summary.init_config_written.as_deref(),
        })),
        SourceSafetyOutputFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        SourceSafetyOutputFormat::Text => Ok(render_source_safety_text(SourceSafetyTextInput {
            source_id: &summary.source_id,
            dataset_id: &summary.dataset_id,
            mode: &summary.mode,
            read_only: Some(summary.read_only),
            score: summary.score,
            grade: summary.grade,
            status: summary.status,
            table_count: Some(summary.table_count),
            unsafe_table_count: Some(summary.unsafe_table_count),
            slot: Some(&summary.slot),
            subscription_conflicts: &summary.subscription_conflicts,
            factors: &summary.factors,
            recommended_actions: &summary.recommended_actions,
            init_recommendation: summary.init_recommendation.as_ref(),
            init_config_written: summary.init_config_written.as_deref(),
        })),
    }
}
