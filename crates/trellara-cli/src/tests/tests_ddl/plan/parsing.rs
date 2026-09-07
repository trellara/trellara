use super::*;

#[test]
fn ddl_plan_rejects_column_change_without_column_identity() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let err = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_nullable_column:public.sales".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect_err("DDL plan should reject malformed column object");

    assert!(err
        .to_string()
        .contains("column-level DDL must use schema.table.column"));
}

#[test]
fn ddl_plan_rejects_relation_change_with_column_identity() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let err = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_table:public.sales.extra".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect_err("DDL plan should reject malformed relation object");

    assert!(err
        .to_string()
        .contains("relation-level DDL must use schema.table"));
}

#[test]
fn ddl_plan_accepts_dash_separated_change_kinds() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add-nullable-column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");

    assert_eq!(
        summary.changes[0].kind,
        DdlPlanChangeKind::AddNullableColumn
    );
    assert_eq!(summary.changes[0].relation.as_deref(), Some("public.sales"));
}
