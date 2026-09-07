use super::*;

#[test]
fn ddl_apply_plan_extracts_safe_target_postgres_statements() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = DdlApplyPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("apply plan");

    assert!(summary.executable);
    assert_eq!(
        summary.transaction_boundary_rule,
        "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release"
    );
    assert_eq!(summary.blocker_count, 0);
    assert_eq!(summary.plan_sha256.len(), 64);
    assert_eq!(summary.statement_count, 1);
    assert_eq!(
        summary.statements[0].sql,
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;"
    );
    assert_eq!(
        summary.statements[0].release_gate_code,
        "post_ddl_dml_release"
    );
    assert_eq!(summary.statements[0].statement_sha256.len(), 64);
    assert_eq!(
        summary.statements[0].transaction_scope,
        "single target schema transaction before barrier ACK"
    );
    assert!(summary.statements[0]
        .release_gate
        .contains("post_ddl_dml_release"));
    assert!(summary
        .steps
        .iter()
        .any(|step| step.contains("one target Postgres schema transaction")));
    let script = summary
        .target_postgres_transaction_script
        .as_ref()
        .expect("target Postgres transaction script");
    assert!(script.contains("BEGIN;"));
    assert!(script.contains("COMMIT;"));
    assert!(script.contains(&summary.plan_sha256));
    assert!(script.contains(&summary.statements[0].statement_sha256));
    assert!(script.contains("Do not record target_postgres ACK"));
    assert_eq!(summary.target_postgres_ack_commands.len(), 1);
    let ack_command = &summary.target_postgres_ack_commands[0];
    assert!(ack_command.contains("schema ddl-barrier ack"));
    assert!(ack_command.contains("--sink target_postgres"));
    assert!(ack_command.contains(&summary.barrier_id));
    assert!(ack_command.contains(&summary.plan_sha256));
    assert!(ack_command.contains(&summary.statements[0].statement_sha256));
    assert!(summary
        .next_commands
        .iter()
        .any(|command| command.contains("schema ddl-barrier record")));
}

#[test]
fn ddl_apply_plan_text_renders_statement_and_plan_digests() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = DdlApplyPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Text,
        },
    )
    .expect("apply plan");

    let output =
        render_ddl_apply_plan_summary(&summary, QuickstartOutputFormat::Text).expect("render");

    assert!(output.contains("plan_sha256="));
    assert!(output.contains("statement_sha256="));
    assert!(output.contains("target_postgres_transaction_script:"));
    assert!(output.contains("target_postgres_ack_commands:"));
    assert!(output.contains("schema ddl-barrier ack"));
    assert!(output.contains("--sink target_postgres"));
    assert!(output.contains("BEGIN;"));
    assert!(output.contains("COMMIT;"));
    assert!(output.contains(&summary.plan_sha256));
    assert!(output.contains(&summary.statements[0].statement_sha256));
}

#[test]
fn ddl_apply_plan_fails_closed_for_blocked_or_unreviewed_changes() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = DdlApplyPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec![
                "add_nullable_column:public.sales.discount_code:text".to_string(),
                "drop_column:public.sales.legacy_code".to_string(),
            ],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("apply plan");

    assert!(!summary.executable);
    assert_eq!(summary.statement_count, 1);
    assert!(summary
        .blockers
        .iter()
        .any(|blocker| blocker.contains("DDL propagation plan is blocked")));
    assert!(summary
        .blockers
        .iter()
        .any(|blocker| blocker.contains("drop_column:public.sales.legacy_code")));
    assert_eq!(
        summary.steps,
        vec!["fix blockers and rerun schema ddl-apply-plan"]
    );
    assert!(summary.target_postgres_transaction_script.is_none());
    assert!(summary.target_postgres_ack_commands.is_empty());
}

#[test]
fn ddl_apply_plan_rejects_type_specs_with_extra_clauses() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = DdlApplyPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["widen_type:public.sales.amount:numeric(10,2) not null".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("apply plan");

    assert!(!summary.executable);
    assert_eq!(summary.statement_count, 0);
    assert!(summary
        .blockers
        .iter()
        .any(|blocker| blocker.contains("has no safe target_postgres_sql")));
}
