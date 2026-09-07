use super::*;

#[test]
fn cli_parses_repair_plan_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "repair",
        "plan",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Repair {
            command: RepairCommand::Plan(_)
        }
    ));
}

#[test]
fn cli_parses_repair_plan_alias_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "repair-plan",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(cli.command, Command::RepairPlan(_)));
}

#[test]
fn cli_parses_source_safety_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "source-safety",
        "--config",
        "examples/retail-fleet/strict.yml",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::SourceSafety(SourceSafetyArgs {
            config: Some(config),
            database_url: None,
            ..
        }) if config.as_path() == Path::new("examples/retail-fleet/strict.yml")
    ));
}

#[test]
fn cli_parses_source_safety_direct_database_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "source-safety",
        "--database-url",
        "postgresql://source/app",
        "--source-id",
        "store-fleet",
        "--dataset-id",
        "sales",
        "--slot",
        "trellara_sales_slot",
        "--target-database-url",
        "postgresql://target/app",
        "--write-init",
        "target/trellara.yml",
        "--force",
        "--format",
        "html",
        "--output",
        "target/source-safety.html",
        "--table",
        "public.sales",
        "--table",
        "public.payments",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::SourceSafety(SourceSafetyArgs {
            config: None,
            database_url: Some(database_url),
            target_database_url: Some(target_database_url),
            source_id,
            dataset_id,
            slot,
            format,
            output: Some(output),
            table,
            write_init: Some(write_init),
            force: true,
            ..
        }) if database_url == "postgresql://source/app"
            && target_database_url == "postgresql://target/app"
            && source_id == "store-fleet"
            && dataset_id == "sales"
            && slot == "trellara_sales_slot"
            && format == SourceSafetyOutputFormat::Html
            && output.as_path() == Path::new("target/source-safety.html")
            && table == vec!["public.sales".to_string(), "public.payments".to_string()]
            && write_init.as_path() == Path::new("target/trellara.yml")
    ));
}
