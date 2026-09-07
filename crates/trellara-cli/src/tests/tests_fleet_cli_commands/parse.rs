use super::*;

#[test]
fn cli_parses_fleet_init_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "fleet",
        "init",
        "--source-database-url",
        "postgresql://source/app",
        "--target-database-url",
        "postgresql://target/app",
        "--dataset",
        "customer-east",
        "--dataset",
        "customer-west",
        "--table",
        "public.sales",
        "--output-dir",
        "target/fleet",
        "--force",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Fleet {
            command: FleetCommand::Init(FleetInitArgs {
                dataset,
                output_dir,
                force: true,
                ..
            })
        } if dataset == vec!["customer-east".to_string(), "customer-west".to_string()]
            && output_dir.as_path() == Path::new("target/fleet")
    ));
}

#[test]
fn cli_parses_fleet_report_command() {
    let cli = parse_two_config_fleet_command("report");

    assert!(matches!(
        cli.command,
        Command::Fleet {
            command: FleetCommand::Report(FleetReportArgs {
                config,
                format: QuickstartOutputFormat::Text,
            })
        } if config == expected_fleet_configs()
    ));
}

#[test]
fn cli_parses_fleet_scorecard_command() {
    let cli = parse_two_config_fleet_command("scorecard");

    assert!(matches!(
        cli.command,
        Command::Fleet {
            command: FleetCommand::Scorecard(FleetScorecardArgs {
                config,
                format: QuickstartOutputFormat::Text,
            })
        } if config == expected_fleet_configs()
    ));
}

#[test]
fn cli_parses_fleet_identity_audit_command() {
    let cli = parse_two_config_fleet_command("identity-audit");

    assert!(matches!(
        cli.command,
        Command::Fleet {
            command: FleetCommand::IdentityAudit(FleetIdentityAuditArgs {
                config,
                format: QuickstartOutputFormat::Text,
            })
        } if config == expected_fleet_configs()
    ));
}

#[test]
fn cli_parses_fleet_evidence_plan_command() {
    let cli = parse_two_config_fleet_command("evidence-plan");

    assert!(matches!(
        cli.command,
        Command::Fleet {
            command: FleetCommand::EvidencePlan(FleetEvidencePlanArgs {
                config,
                format: QuickstartOutputFormat::Text,
            })
        } if config == expected_fleet_configs()
    ));
}

#[test]
fn cli_parses_fleet_control_plane_command() {
    let cli = parse_two_config_fleet_command("control-plane");

    assert!(matches!(
        cli.command,
        Command::Fleet {
            command: FleetCommand::ControlPlane(FleetControlPlaneArgs {
                config,
                format: QuickstartOutputFormat::Text,
            })
        } if config == expected_fleet_configs()
    ));
}

fn parse_two_config_fleet_command(subcommand: &str) -> Cli {
    Cli::try_parse_from([
        "trellara",
        "fleet",
        subcommand,
        "--config",
        "examples/retail-fleet/local.yml",
        "--config",
        "examples/retail-fleet/partitioned.yml",
        "--format",
        "text",
    ])
    .expect("cli parse")
}

fn expected_fleet_configs() -> Vec<PathBuf> {
    vec![
        PathBuf::from("examples/retail-fleet/local.yml"),
        PathBuf::from("examples/retail-fleet/partitioned.yml"),
    ]
}
