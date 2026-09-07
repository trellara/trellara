use super::*;

#[test]
fn cli_parses_dev_up_command() {
    let cli = Cli::try_parse_from(["trellara", "dev", "up"]).expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Dev {
            command: DevCommand::Up(DevStackArgs {
                dry_run: false,
                with_kafka: false,
                runtime: false,
            })
        }
    ));
}

#[test]
fn cli_parses_dev_up_runtime_dry_run_command() {
    let cli = Cli::try_parse_from(["trellara", "dev", "up", "--runtime", "--dry-run"])
        .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Dev {
            command: DevCommand::Up(DevStackArgs {
                dry_run: true,
                with_kafka: false,
                runtime: true,
            })
        }
    ));
}
