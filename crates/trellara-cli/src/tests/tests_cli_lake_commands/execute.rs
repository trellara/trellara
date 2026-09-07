use super::*;

#[tokio::test]
async fn lake_spark_template_command_renders_sql_and_json() {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-spark-template-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("strict.yml");
    fs::create_dir_all(&root).expect("create lake spark template temp dir");
    fs::write(&config_path, STRICT_YAML).expect("write config");

    let sql = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::SparkTemplate {
                command: LakeSparkTemplateCommand::CurrentState(LakeSparkTemplateArgs {
                    config: config_path.clone(),
                    catalog: "spark_catalog".to_string(),
                    namespace: None,
                    table: Some("public.sales".to_string()),
                    epoch_id: "epoch-1".to_string(),
                    target_table: None,
                    primary_key_column: None,
                    accept_complete_with_gaps: true,
                    unsafe_allow_non_consumable_epoch: false,
                    unsafe_override_reason: None,
                    format: QuickstartOutputFormat::Text,
                }),
            },
        },
    })
    .await
    .expect("spark SQL output");
    assert!(sql.starts_with("-- Trellara Spark template: raw CDC to current-state table."));
    assert!(sql.contains("MERGE INTO spark_catalog.retail_sales"));
    assert!(!sql.contains("${"));

    let json = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::SparkTemplate {
                command: LakeSparkTemplateCommand::Scd2(LakeSparkTemplateArgs {
                    config: config_path,
                    catalog: "spark_catalog".to_string(),
                    namespace: None,
                    table: Some("public.sales".to_string()),
                    epoch_id: "epoch-1".to_string(),
                    target_table: None,
                    primary_key_column: None,
                    accept_complete_with_gaps: false,
                    unsafe_allow_non_consumable_epoch: false,
                    unsafe_override_reason: None,
                    format: QuickstartOutputFormat::Json,
                }),
            },
        },
    })
    .await
    .expect("spark JSON output");
    assert!(json.contains("\"template\": \"scd2\""));
    assert!(json.contains("\"target_table\": \"retail_sales__public__sales__scd2\""));
    assert!(json.contains("\"sql\""));

    let dashboard = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::SparkTemplate {
                command: LakeSparkTemplateCommand::Dashboard(LakeSparkTemplateArgs {
                    config: root.join("strict.yml"),
                    catalog: "spark_catalog".to_string(),
                    namespace: None,
                    table: Some("public.sales".to_string()),
                    epoch_id: "epoch-1".to_string(),
                    target_table: None,
                    primary_key_column: None,
                    accept_complete_with_gaps: true,
                    unsafe_allow_non_consumable_epoch: false,
                    unsafe_override_reason: None,
                    format: QuickstartOutputFormat::Text,
                }),
            },
        },
    })
    .await
    .expect("dashboard SQL output");
    assert!(dashboard.contains("epoch_release_gate"));
    assert!(dashboard.contains("retail_sales__trellara__fanin___trellara_epoch_sources"));
    assert!(!dashboard.contains("${"));

    fs::remove_dir_all(root).expect("remove lake spark template temp dir");
}
