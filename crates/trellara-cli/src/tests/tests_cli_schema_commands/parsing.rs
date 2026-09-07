use super::*;

#[test]
fn cli_parses_schema_discover_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "schema",
        "discover",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Schema {
            command: SchemaCommand::Discover(_)
        }
    ));
}

#[test]
fn cli_parses_schema_discover_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "schema-discover",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::SchemaDiscover(_)));
}

#[test]
fn cli_parses_schema_ddl_plan_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "schema",
        "ddl-plan",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--change",
        "add_nullable_column:public.sales.discount_code:text",
        "--apply-mode",
        "auto-safe",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Schema {
            command: SchemaCommand::DdlPlan(DdlPlanArgs {
                apply_mode: DdlPlanApplyMode::AutoSafe,
                format: QuickstartOutputFormat::Text,
                ..
            })
        }
    ));
}

#[test]
fn cli_parses_schema_ddl_apply_plan_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "schema",
        "ddl-apply-plan",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--change",
        "add_nullable_column:public.sales.discount_code:text",
        "--apply-mode",
        "auto-safe",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Schema {
            command: SchemaCommand::DdlApplyPlan(DdlPlanArgs {
                apply_mode: DdlPlanApplyMode::AutoSafe,
                format: QuickstartOutputFormat::Text,
                ..
            })
        }
    ));
}

#[test]
fn cli_parses_schema_ddl_envelope_plan_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "schema",
        "ddl-envelope-plan",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--file",
        "sample-envelope.pb",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Schema {
            command: SchemaCommand::DdlEnvelopePlan(DdlEnvelopePlanArgs {
                format: QuickstartOutputFormat::Text,
                ..
            })
        }
    ));
}
