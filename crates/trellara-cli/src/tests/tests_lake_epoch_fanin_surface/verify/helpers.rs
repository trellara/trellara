use super::*;

pub(super) fn verify_summary(
    scenario: LakeEpochScenario,
    label: &str,
    mutate_stream: impl FnOnce(&mut LakeEpochSummary),
    mutate_lake: impl FnOnce(&mut LakeEpochSummary),
) -> LakeFaninVerifySummary {
    verify_summary_with_gap_acceptance(scenario, label, false, mutate_stream, mutate_lake)
}

pub(super) fn verify_summary_with_gap_acceptance(
    scenario: LakeEpochScenario,
    label: &str,
    accept_complete_with_gaps: bool,
    mutate_stream: impl FnOnce(&mut LakeEpochSummary),
    mutate_lake: impl FnOnce(&mut LakeEpochSummary),
) -> LakeFaninVerifySummary {
    let root = std::env::temp_dir().join(format!(
        "trellara-lake-fanin-verify-{label}-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create lake fanin verify temp dir");
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let args = lake_epoch_test_args(scenario);
    let mut stream = LakeEpochSummary::from_config(&config, &args);
    mutate_stream(&mut stream);
    let mut lake = stream.clone();
    mutate_lake(&mut lake);

    let stream_epoch = root.join("stream-epoch.json");
    let lake_epoch = root.join("lake-epoch.json");
    fs::write(
        &stream_epoch,
        serde_json::to_string_pretty(&stream).expect("serialize stream epoch"),
    )
    .expect("write stream epoch");
    fs::write(
        &lake_epoch,
        serde_json::to_string_pretty(&lake).expect("serialize lake epoch"),
    )
    .expect("write lake epoch");

    let summary = LakeFaninVerifySummary::from_args(
        &config,
        &LakeFaninVerifyArgs {
            config: PathBuf::from("test.yml"),
            stream_epoch,
            lake_epoch,
            accept_complete_with_gaps,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("verify");

    fs::remove_dir_all(root).expect("remove lake fanin verify temp dir");
    summary
}

pub(super) fn assert_matches_without_mismatches(summary: &LakeFaninVerifySummary) {
    assert_eq!(summary.status, LakeFaninVerifyStatus::Match);
    assert_eq!(summary.mismatch_count, 0);
    assert_eq!(summary.warning_mismatch_count, 0);
    assert_eq!(summary.blocker_mismatch_count, 0);
    assert!(summary.spark_consumption_allowed);
}

pub(super) fn assert_blocker(summary: &LakeFaninVerifySummary, field: &str) {
    assert_eq!(summary.status, LakeFaninVerifyStatus::Blocked);
    assert!(!summary.spark_consumption_allowed);
    assert!(summary.blocker_mismatch_count > 0);
    assert!(summary.mismatches.iter().any(|mismatch| {
        mismatch.field == field && mismatch.severity == LakeFaninVerifyMismatchSeverity::Blocker
    }));
}

pub(super) fn refresh_manifest_digest(epoch: &mut LakeEpochSummary) {
    epoch.manifest_digest = lake_epoch_summary_manifest_digest(epoch);
}
