use super::*;

#[tokio::test]
async fn schema_ddl_plan_command_renders_text_boundary_review() {
    let root = std::env::temp_dir().join(format!(
        "trellara-schema-ddl-plan-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("strict.yml");
    fs::create_dir_all(&root).expect("create ddl plan temp dir");
    fs::write(&config_path, STRICT_YAML).expect("write config");

    let output = execute(Cli {
        command: Command::Schema {
            command: SchemaCommand::DdlPlan(DdlPlanArgs {
                config: config_path,
                changes: vec![
                    "add_nullable_column:public.sales.discount_code:text".to_string(),
                    "rename_column:public.sales.total:amount".to_string(),
                ],
                apply_mode: DdlPlanApplyMode::ManualReview,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("ddl plan output");

    assert!(output.contains("Trellara schema DDL propagation plan"));
    assert!(output.contains("verdict=requires_manual_review"));
    assert!(output.contains("transaction_boundary_rule:"));
    assert!(output.contains("propagation: barrier_id=ddl-barrier-"));
    assert!(output.contains("required_acks=3"));
    assert!(output.contains("ack_quorum=all_required_sinks"));
    assert!(output.contains("policy_modes:"));
    assert!(output.contains("manual_approval_required active=true changes=2"));
    assert!(output.contains("shadow_plan_only active=false changes=0"));
    assert!(output.contains("sinks:"));
    assert!(output.contains("kind=target_postgres"));
    assert!(output.contains("kind=raw_cdc_lake"));
    assert!(output.contains("phases:"));
    assert!(output.contains("release_gates:"));
    assert!(output.contains("schema_barrier_recorded"));
    assert!(output.contains("post_ddl_dml_release"));
    assert!(output.contains("release_blockers"));
    assert!(output.contains("add_nullable_column"));
    assert!(output.contains("rename_column"));
    assert!(output.contains("schema barrier"));
    assert!(output.contains("action[target_postgres]:"));
    assert!(output.contains("trellara schema ddl-barrier record --config"));
    assert!(output.contains("--change add_nullable_column:public.sales.discount_code:text"));
    assert!(output.contains("--change rename_column:public.sales.total:amount"));
    assert!(output.contains("--apply-mode manual-review"));
    assert!(output.contains("trellara schema ddl-barrier ack --config"));
    assert!(output.contains("trellara schema ddl-barrier status --config"));
    assert!(!output.contains("target_postgres_sql:"));

    fs::remove_dir_all(root).expect("remove ddl plan temp dir");
}

#[tokio::test]
async fn schema_ddl_apply_plan_command_renders_safe_text_plan() {
    let root = std::env::temp_dir().join(format!(
        "trellara-schema-ddl-apply-plan-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("strict.yml");
    fs::create_dir_all(&root).expect("create ddl apply plan temp dir");
    fs::write(&config_path, STRICT_YAML).expect("write config");

    let output = execute(Cli {
        command: Command::Schema {
            command: SchemaCommand::DdlApplyPlan(DdlPlanArgs {
                config: config_path,
                changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
                apply_mode: DdlPlanApplyMode::AutoSafe,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("ddl apply plan output");

    assert!(output.contains("Trellara target Postgres DDL apply plan"));
    assert!(output.contains("executable=true statements=1 blockers=0"));
    assert!(output.contains(
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;"
    ));
    assert!(output.contains("post_ddl_dml_release"));
    assert!(output.contains("target_postgres_ack_commands:"));
    assert!(output.contains("--plan-sha256"));
    assert!(output.contains("--statement-sha256"));
    assert!(output.contains("schema ddl-barrier record"));
    assert!(output.contains("schema ddl-barrier status"));

    fs::remove_dir_all(root).expect("remove ddl apply plan temp dir");
}

#[tokio::test]
async fn schema_ddl_plan_command_renders_auto_safe_target_sql_preview() {
    let root = std::env::temp_dir().join(format!(
        "trellara-schema-ddl-sql-preview-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("strict.yml");
    fs::create_dir_all(&root).expect("create ddl sql preview temp dir");
    fs::write(&config_path, STRICT_YAML).expect("write config");

    let output = execute(Cli {
        command: Command::Schema {
            command: SchemaCommand::DdlPlan(DdlPlanArgs {
                config: config_path,
                changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
                apply_mode: DdlPlanApplyMode::AutoSafe,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("ddl sql preview output");

    assert!(output.contains(
        "target_postgres_sql: ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;"
    ));
    assert!(output.contains("action[target_postgres]: apply compatible add_nullable_column"));
    assert!(output.contains("trellara schema ddl-barrier record --config"));
    assert!(output.contains("--change add_nullable_column:public.sales.discount_code:text"));
    assert!(output.contains("--apply-mode auto-safe"));
    assert!(output.contains("trellara schema ddl-barrier ack --config"));
    assert!(output.contains("trellara schema ddl-barrier status --config"));

    fs::remove_dir_all(root).expect("remove ddl sql preview temp dir");
}
