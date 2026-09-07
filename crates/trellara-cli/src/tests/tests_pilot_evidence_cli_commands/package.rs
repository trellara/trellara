use super::*;

#[test]
fn cli_parses_pilot_package_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot",
        "package",
        "--config",
        "examples/retail-fleet/local.yml",
        "--output",
        "target/pilot-package",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Pilot {
            command: PilotCommand::Package(PilotPackageArgs { output, .. })
        } if output.as_path() == Path::new("target/pilot-package")
    ));
}

#[test]
fn cli_parses_pilot_package_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "pilot-package",
        "--config",
        "examples/retail-fleet/local.yml",
        "--output",
        "target/pilot-package",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::PilotPackage(PilotPackageArgs { output, .. })
            if output.as_path() == Path::new("target/pilot-package")
    ));
}
