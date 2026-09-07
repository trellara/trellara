use super::*;

mod commands;
mod summary;
mod verify;

fn lake_epoch_test_args(scenario: LakeEpochScenario) -> LakeEpochArgs {
    LakeEpochArgs {
        config: PathBuf::from("test.yml"),
        scenario,
        required_source_count: 12,
        offline_source_count: 3,
        duplicate_replay_count: 2,
        format: QuickstartOutputFormat::Json,
    }
}
