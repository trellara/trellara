use super::*;

#[test]
fn cli_parses_lake_inspect_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "inspect",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--file",
        "tx.pb",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::Inspect(LakeInspectArgs { file, .. })
        } if file.as_path() == Path::new("tx.pb")
    ));
}
