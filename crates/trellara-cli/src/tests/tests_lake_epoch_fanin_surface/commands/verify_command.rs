use super::*;

#[tokio::test]
async fn lake_fanin_verify_command_renders_text_report() {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-fanin-verify-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create lake fanin verify command temp dir");
    let config_path = root.join("strict.yml");
    fs::write(&config_path, STRICT_YAML).expect("write config");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");
    let epoch = LakeEpochSummary::from_config(
        &config,
        &lake_epoch_test_args(LakeEpochScenario::LateStoreRecoveryCompletesEpoch),
    );
    let stream_epoch = root.join("stream-epoch.json");
    let lake_epoch = root.join("lake-epoch.json");
    let epoch_json = serde_json::to_string_pretty(&epoch).expect("serialize epoch");
    fs::write(&stream_epoch, &epoch_json).expect("write stream epoch");
    fs::write(&lake_epoch, &epoch_json).expect("write lake epoch");

    let output = execute(Cli {
        command: Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Verify(LakeFaninVerifyArgs {
                    config: config_path,
                    stream_epoch,
                    lake_epoch,
                    accept_complete_with_gaps: false,
                    format: QuickstartOutputFormat::Text,
                }),
            },
        },
    })
    .await
    .expect("lake fanin verify output");

    assert!(output.contains("Trellara lake fan-in verification"));
    assert!(output.contains("status=match"));
    assert!(output.contains("spark_consumption_allowed=true"));
    assert!(output.contains("source_counts: match=true"));
    assert!(output.contains("stream_required=12 stream_complete=12"));
    assert!(output.contains("lake_required=12 lake_complete=12"));
    assert!(output.contains("checksum_rollup: match=true"));
    assert!(output.contains("checks: matched=28 mismatches=0 warnings=0 blockers=0"));

    fs::remove_dir_all(root).expect("remove lake fanin verify command temp dir");
}
