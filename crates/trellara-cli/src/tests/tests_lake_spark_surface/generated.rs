use super::*;

mod materialization;
mod operations;

fn spark_template_args(epoch_id: &str, accept_complete_with_gaps: bool) -> LakeSparkTemplateArgs {
    LakeSparkTemplateArgs {
        config: PathBuf::from("examples/retail-fleet/strict.yml"),
        catalog: "spark_catalog".to_string(),
        namespace: None,
        table: Some("public.sales".to_string()),
        epoch_id: epoch_id.to_string(),
        target_table: None,
        primary_key_column: None,
        accept_complete_with_gaps,
        unsafe_allow_non_consumable_epoch: false,
        unsafe_override_reason: None,
        format: QuickstartOutputFormat::Text,
    }
}

fn assert_policy_guarded_gap_gate(sql: &str, accept_value: &str) {
    assert!(sql.contains("e.state = 'complete_with_gaps'"));
    assert!(sql.contains("e.policy = 'publish_with_gaps'"));
    assert!(sql.contains(&format!("{accept_value} = true")));
}
