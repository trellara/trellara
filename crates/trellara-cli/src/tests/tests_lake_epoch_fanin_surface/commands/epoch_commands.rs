use super::*;

#[tokio::test]
async fn lake_epoch_command_renders_text_decision() {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-epoch-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create lake epoch command temp dir");
    let config_path = root.join("strict.yml");
    fs::write(&config_path, STRICT_YAML).expect("write config");

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::Epoch(LakeEpochArgs {
                config: config_path.clone(),
                scenario: LakeEpochScenario::OfflineStoresPublishWithGaps,
                required_source_count: 12,
                offline_source_count: 3,
                duplicate_replay_count: 2,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("lake epoch output");

    assert!(output.contains("Trellara lake epoch"));
    assert!(output.contains("dataset: retail-sales"));
    assert!(output.contains("state=complete_with_gaps"));
    assert!(output.contains("watermarks: global_low=0/16B6C60 max_source=0/16B6CE0"));
    assert!(output.contains(
        "partition_skew: participating=8 total_events=9 min_events=1 max_events=2 ratio_bp=20000 hottest=0 coolest=1,2,3,4,5,6,7"
    ));
    assert!(output.contains("source_watermarks:"));
    assert!(output.contains("store-0012 state=missing"));
    assert!(output.contains("gap_reason=required source missing from published gap epoch"));
    assert!(output.contains("table_rollups:"));
    assert!(output.contains("public.sales transactions=9 changes=9"));
    assert!(output.contains("partition_rollups:"));
    assert!(output.contains(
        "source=store-0001 partition=0 first_commit_lsn=0/16B6C60 last_commit_lsn=0/16B6C60"
    ));
    assert!(output.contains("quarantine_entries:"));
    assert!(output.contains("straggler_policy: publish_with_gaps"));
    assert!(output.contains("manifest_digest: "));
    assert!(output.contains("requires_explicit_gap_acceptance"));
    assert!(output.contains("accept_complete_with_gaps=true"));

    fs::remove_dir_all(root).expect("remove lake epoch command temp dir");
}

#[tokio::test]
async fn lake_fanin_epoch_spec_command_renders_epoch_contract() {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-fanin-epoch-spec-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create lake fanin epoch spec command temp dir");
    let config_path = root.join("strict.yml");
    fs::write(&config_path, STRICT_YAML).expect("write config");

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::EpochSpec(LakeEpochArgs {
                    config: config_path,
                    scenario: LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
                    required_source_count: 12,
                    offline_source_count: 3,
                    duplicate_replay_count: 2,
                    format: QuickstartOutputFormat::Text,
                }),
            },
        },
    })
    .await
    .expect("lake fanin epoch spec output");

    assert!(output.contains("Trellara lake epoch"));
    assert!(output.contains("dataset: retail-sales"));
    assert!(output.contains("state=complete"));
    assert!(output.contains("decision: safe_for_default_spark_consumption"));
    assert!(output.contains("source_watermarks:"));
    assert!(output.contains("partition_rollups:"));
    assert!(output.contains("manifest_digest: "));

    fs::remove_dir_all(root).expect("remove lake fanin epoch spec command temp dir");
}
