use super::*;

#[tokio::test]
async fn lake_fanin_plan_command_renders_lake_plan_contract() {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-fanin-plan-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create lake fanin plan command temp dir");
    let config_path = root.join("strict.yml");
    fs::write(&config_path, STRICT_YAML).expect("write config");

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Plan(ConfigArgs {
                    config: config_path,
                }),
            },
        },
    })
    .await
    .expect("lake fanin plan output");
    let summary: serde_json::Value = serde_json::from_str(&output).expect("lake plan json");

    assert_eq!(summary["dataset_id"].as_str(), Some("retail-sales"));
    assert_eq!(
        summary["contract"].as_str(),
        Some("fleet_fanin_append_only_raw_cdc_with_epoch_completeness")
    );
    assert!(summary["fanin_mode"]
        .as_str()
        .is_some_and(|mode| mode.contains("strict")));
    assert!(summary["materializations"]
        .as_array()
        .expect("materializations array")
        .iter()
        .any(|materialization| materialization["kind"] == "epoch_metadata"));
    assert!(summary["epoch_metadata_tables"]
        .as_array()
        .expect("epoch metadata tables")
        .iter()
        .any(|table| table == "_trellara_epoch_partitions"));

    fs::remove_dir_all(root).expect("remove lake fanin plan command temp dir");
}

#[tokio::test]
async fn lake_fanin_ddl_command_renders_raw_cdc_and_metadata_contract() {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-fanin-ddl-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create lake fanin ddl command temp dir");
    let config_path = root.join("strict.yml");
    fs::write(&config_path, STRICT_YAML).expect("write config");

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Ddl(ConfigArgs {
                    config: config_path,
                }),
            },
        },
    })
    .await
    .expect("lake fanin ddl output");
    let summary: serde_json::Value = serde_json::from_str(&output).expect("lake ddl json");

    assert_eq!(summary["dataset_id"].as_str(), Some("retail-sales"));
    assert_eq!(
        summary["contract"].as_str(),
        Some("fleet_fanin_append_only_raw_cdc_with_epoch_completeness")
    );
    assert!(summary["tables"]
        .as_array()
        .expect("ddl tables")
        .iter()
        .any(|table| table["materialization"] == "raw_cdc_append_only"
            && table["ddl"]
                .as_str()
                .is_some_and(|ddl| ddl.contains("__trellara_idempotency_key"))));
    assert!(summary["tables"]
        .as_array()
        .expect("ddl tables")
        .iter()
        .any(|table| table["materialization"] == "epoch_source_metadata"
            && table["table_name"] == "retail_sales__trellara__fanin___trellara_epoch_sources"));
    assert!(summary["tables"]
        .as_array()
        .expect("ddl tables")
        .iter()
        .any(
            |table| table["materialization"] == "epoch_partition_metadata"
                && table["table_name"]
                    == "retail_sales__trellara__fanin___trellara_epoch_partitions"
        ));

    fs::remove_dir_all(root).expect("remove lake fanin ddl command temp dir");
}
