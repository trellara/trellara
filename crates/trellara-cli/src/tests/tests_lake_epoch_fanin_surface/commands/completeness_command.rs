use super::*;

#[tokio::test]
async fn lake_fanin_completeness_command_blocks_gap_epoch_without_acceptance() {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-fanin-completeness-gap-blocked-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create lake fanin completeness command temp dir");
    let config_path = root.join("strict.yml");
    fs::write(&config_path, STRICT_YAML).expect("write config");
    let (stream_epoch, lake_epoch) = write_matching_epoch_artifacts(
        &root,
        &config_path,
        LakeEpochScenario::OfflineStoresPublishWithGaps,
    );

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Completeness(LakeFaninCompletenessArgs {
                    config: config_path,
                    stream_epoch,
                    lake_epoch,
                    accept_complete_with_gaps: false,
                    format: QuickstartOutputFormat::Json,
                }),
            },
        },
    })
    .await
    .expect("lake fanin completeness output");
    let summary: serde_json::Value =
        serde_json::from_str(&output).expect("lake fanin completeness json");

    assert_eq!(
        summary["contract"].as_str(),
        Some("fleet_fanin_append_only_raw_cdc_with_epoch_completeness")
    );
    assert_eq!(
        summary["positioning"].as_str(),
        Some(
            "prove queryable epoch completeness with append-only raw CDC and metadata gates; do not position this as a generic Iceberg sink"
        )
    );
    assert_eq!(
        summary["decision"].as_str(),
        Some("blocked_needs_gap_acceptance")
    );
    assert_eq!(
        summary["completeness_state"].as_str(),
        Some("complete_with_gaps")
    );
    assert_eq!(summary["spark_consumption_allowed"].as_bool(), Some(false));
    assert_eq!(summary["source_state_counts"]["missing"].as_u64(), Some(3));
    assert!(summary["epoch_metadata_tables"]
        .as_array()
        .expect("epoch metadata tables")
        .iter()
        .any(|table| table["table_name"]
            .as_str()
            .is_some_and(|name| name.ends_with("___trellara_verification"))));
    assert!(summary["raw_cdc_tables"]
        .as_array()
        .expect("raw cdc tables")
        .iter()
        .any(|table| table["materialization"] == "raw_cdc_append_only"));
    assert!(summary["deferred_sink_work"]
        .as_array()
        .expect("deferred sink work")
        .iter()
        .any(|item| item
            .as_str()
            .is_some_and(|value| value.contains("generic single-source"))));
    assert!(summary["recovery_path"]
        .as_array()
        .expect("recovery path")
        .iter()
        .any(|step| step
            .as_str()
            .is_some_and(|value| value.contains("--accept-complete-with-gaps"))));

    fs::remove_dir_all(root).expect("remove lake fanin completeness command temp dir");
}

#[tokio::test]
async fn lake_fanin_completeness_command_releases_gap_epoch_with_acceptance() {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-fanin-completeness-gap-accepted-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create lake fanin completeness command temp dir");
    let config_path = root.join("strict.yml");
    fs::write(&config_path, STRICT_YAML).expect("write config");
    let (stream_epoch, lake_epoch) = write_matching_epoch_artifacts(
        &root,
        &config_path,
        LakeEpochScenario::OfflineStoresPublishWithGaps,
    );

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Completeness(LakeFaninCompletenessArgs {
                    config: config_path,
                    stream_epoch,
                    lake_epoch,
                    accept_complete_with_gaps: true,
                    format: QuickstartOutputFormat::Text,
                }),
            },
        },
    })
    .await
    .expect("lake fanin completeness output");

    assert!(output.contains("Trellara lake fan-in completeness"));
    assert!(output.contains("decision=ready_with_accepted_gaps"));
    assert!(output.contains("accepted_gaps=true"));
    assert!(output.contains("spark_consumption_allowed=true"));
    assert!(output.contains("raw_cdc_tables:"));
    assert!(output.contains("epoch_metadata_tables:"));
    assert!(output.contains("spark-completeness-dashboard.sql"));
    assert!(output.contains("trellara lake fanin verify"));
    assert!(output.contains("deferred_sink_work:"));
    assert!(output.contains("native Rust current-state Iceberg upserts or deletes"));
    assert!(output.contains("verification: status=match"));

    fs::remove_dir_all(root).expect("remove lake fanin completeness command temp dir");
}

fn write_matching_epoch_artifacts(
    root: &Path,
    config_path: &Path,
    scenario: LakeEpochScenario,
) -> (PathBuf, PathBuf) {
    let config = TrellaraConfig::from_path(config_path).expect("parse config");
    let epoch = LakeEpochSummary::from_config(&config, &lake_epoch_test_args(scenario));
    let epoch_json = serde_json::to_string_pretty(&epoch).expect("serialize epoch");
    let stream_epoch = root.join("stream-epoch.json");
    let lake_epoch = root.join("lake-epoch.json");
    fs::write(&stream_epoch, &epoch_json).expect("write stream epoch");
    fs::write(&lake_epoch, epoch_json).expect("write lake epoch");
    (stream_epoch, lake_epoch)
}
