use super::*;

#[test]
fn cli_parses_lake_plan_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "plan",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Plan(_)
        }
    ));
}

#[test]
fn cli_parses_lake_ddl_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "ddl",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Ddl(_)
        }
    ));
}

#[test]
fn cli_parses_lake_writer_plan_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "writer-plan",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--file",
        "first.pb",
        "--file",
        "second.pb",
        "--epoch-id",
        "epoch-2026-08-16T06",
        "--source-bucket-count",
        "8",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::WriterPlan(LakeWriterPlanArgs {
                epoch_id,
                source_bucket_count: 8,
                format: QuickstartOutputFormat::Text,
                ..
            })
        } if epoch_id == "epoch-2026-08-16T06"
    ));
}

#[test]
fn cli_defaults_lake_writer_plan_to_deterministic_epoch_id() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "writer-plan",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--file",
        "first.pb",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::WriterPlan(LakeWriterPlanArgs {
                epoch_id,
                ..
            })
        } if epoch_id == DETERMINISTIC_EPOCH_ID
    ));
}

#[test]
fn cli_parses_lake_fanin_run_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "fanin",
        "run",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--file",
        "first.pb",
        "--file",
        "second.pb",
        "--epoch-id",
        "epoch-2026-08-16T06",
        "--source-bucket-count",
        "4",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Run(LakeFaninRunArgs {
                    epoch_id,
                    source_bucket_count: 4,
                    format: QuickstartOutputFormat::Text,
                    ..
                })
            }
        } if epoch_id == "epoch-2026-08-16T06"
    ));
}

#[test]
fn cli_defaults_lake_fanin_run_to_deterministic_epoch_id() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "fanin",
        "run",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--file",
        "first.pb",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Fanin {
                command: LakeFaninCommand::Run(LakeFaninRunArgs {
                    epoch_id,
                    ..
                })
            }
        } if epoch_id == DETERMINISTIC_EPOCH_ID
    ));
}
