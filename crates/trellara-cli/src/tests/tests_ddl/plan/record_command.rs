use super::*;

#[test]
fn ddl_plan_record_command_preserves_all_staged_changes_and_apply_mode() {
    let yaml = STRICT_YAML.replace(
            "  mode: strict_transaction_order",
            "  mode: partitioned_scale_mode\n  unknown_table_policy: allow_compatible\n  partition:\n    partition_count: 4\n    key_column: store_id",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("partitioned.yml"),
            changes: vec![
                "add_table:public.refunds".to_string(),
                "add_nullable_column:public.sales.discount_code:text".to_string(),
            ],
            apply_mode: DdlPlanApplyMode::StagedRollout,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");

    assert_eq!(summary.verdict, DdlPlanVerdict::ReadyToApply);
    let record_command = summary
        .next_commands
        .iter()
        .find(|command| command.contains("schema ddl-barrier record"))
        .expect("record command");
    assert!(record_command.contains("--change add_table:public.refunds"));
    assert!(record_command.contains("--change add_nullable_column:public.sales.discount_code:text"));
    assert!(record_command.contains("--apply-mode staged-rollout"));
    assert!(record_command.contains("--barrier-lsn <ddl-lsn>"));
    assert!(record_command.contains("--schema-version <schema-version>"));
}
