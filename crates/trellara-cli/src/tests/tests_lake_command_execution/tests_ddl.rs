use super::*;

#[tokio::test]
async fn lake_ddl_command_renders_materialization_tables() {
    let config_file = std::env::temp_dir().join(format!(
        "trellara-lake-ddl-config-{}-{}.yml",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::write(&config_file, STRICT_YAML).expect("write lake ddl config");

    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "ddl",
        "--config",
        config_file.to_str().expect("utf8 config path"),
    ])
    .expect("cli parse");

    let output = execute(cli).await.expect("execute");

    assert!(output.contains("\"table_count\": 7"));
    assert!(output
        .contains("\"contract\": \"fleet_fanin_append_only_raw_cdc_with_epoch_completeness\""));
    assert!(output.contains("\"materialization\": \"raw_cdc_append_only\""));
    assert!(output.contains("\"materialization\": \"epoch_metadata\""));
    assert!(output.contains("\"materialization\": \"epoch_source_metadata\""));
    assert!(output.contains("\"materialization\": \"epoch_partition_metadata\""));
    assert!(output.contains("retail_sales__public__sales__raw_cdc"));
    assert!(output.contains("retail_sales__trellara__fanin___trellara_epochs"));
    assert!(output.contains("retail_sales__trellara__fanin___trellara_epoch_partitions"));
    assert!(output.contains("iceberg_snapshot_id TEXT NOT NULL"));
    assert!(output.contains("raw_table_snapshot_ids_json TEXT NOT NULL"));
    assert!(output.contains("unpartitioned_l1_metadata_release_marker"));
    assert!(output.contains("manifest_digest TEXT NOT NULL"));
    assert!(output.contains("start_lsn TEXT NOT NULL"));
    assert!(output.contains("end_lsn TEXT NOT NULL"));
    assert!(output.contains("__trellara_commit_lsn"));
    assert!(output.contains("__trellara_epoch_id"));
    assert!(output.contains("source transaction envelope is the visibility boundary"));
    assert!(output.contains("\"checkpoint_contract\""));

    fs::remove_file(config_file).expect("remove config");
}
