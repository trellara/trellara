use super::*;

#[test]
fn cli_parses_lake_epoch_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "epoch",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--scenario",
        "conflicting-duplicate-quarantine",
        "--required-source-count",
        "20",
        "--offline-source-count",
        "4",
        "--duplicate-replay-count",
        "3",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Epoch(LakeEpochArgs {
                scenario: LakeEpochScenario::ConflictingDuplicateQuarantine,
                required_source_count: 20,
                offline_source_count: 4,
                duplicate_replay_count: 3,
                format: QuickstartOutputFormat::Text,
                ..
            })
        }
    ));
}

#[test]
fn cli_parses_lake_fanin_verify_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "fanin",
        "verify",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--stream-epoch",
        "stream-epoch.json",
        "--lake-epoch",
        "lake-epoch.json",
        "--accept-complete-with-gaps",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Verify(LakeFaninVerifyArgs {
                    stream_epoch,
                    lake_epoch,
                    accept_complete_with_gaps: true,
                    format: QuickstartOutputFormat::Text,
                    ..
                })
            }
        } if stream_epoch.as_path() == Path::new("stream-epoch.json")
            && lake_epoch.as_path() == Path::new("lake-epoch.json")
    ));
}

#[test]
fn cli_parses_lake_fanin_completeness_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "fanin",
        "completeness",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--stream-epoch",
        "stream-epoch.json",
        "--lake-epoch",
        "lake-epoch.json",
        "--accept-complete-with-gaps",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Completeness(LakeFaninCompletenessArgs {
                    stream_epoch,
                    lake_epoch,
                    accept_complete_with_gaps: true,
                    format: QuickstartOutputFormat::Text,
                    ..
                })
            }
        } if stream_epoch.as_path() == Path::new("stream-epoch.json")
            && lake_epoch.as_path() == Path::new("lake-epoch.json")
    ));
}

#[test]
fn cli_parses_lake_fanin_plan_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "fanin",
        "plan",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Plan(ConfigArgs { config })
            }
        } if config.as_path() == Path::new("examples/retail-fleet/strict.yml")
    ));
}

#[test]
fn cli_parses_lake_fanin_ddl_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "fanin",
        "ddl",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Ddl(ConfigArgs { config })
            }
        } if config.as_path() == Path::new("examples/retail-fleet/strict.yml")
    ));
}

#[test]
fn cli_parses_lake_fanin_epoch_spec_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "fanin",
        "epoch-spec",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--scenario",
        "late-store-recovery-completes-epoch",
        "--required-source-count",
        "12",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::EpochSpec(LakeEpochArgs {
                    scenario: LakeEpochScenario::LateStoreRecoveryCompletesEpoch,
                    required_source_count: 12,
                    format: QuickstartOutputFormat::Text,
                    ..
                })
            }
        }
    ));
}
