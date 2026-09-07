use super::*;

#[test]
fn cli_parses_lake_spark_template_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "spark-template",
        "current-state",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--table",
        "public.sales",
        "--epoch-id",
        "epoch-1",
        "--accept-complete-with-gaps",
        "--unsafe-allow-non-consumable-epoch",
        "--unsafe-override-reason",
        "incident replay window",
        "--format",
        "json",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::SparkTemplate {
                command: LakeSparkTemplateCommand::CurrentState(LakeSparkTemplateArgs {
                    table: Some(table),
                    epoch_id,
                    accept_complete_with_gaps: true,
                    unsafe_allow_non_consumable_epoch: true,
                    unsafe_override_reason: Some(reason),
                    format: QuickstartOutputFormat::Json,
                    ..
                })
            }
        } if table == "public.sales" && epoch_id == "epoch-1" && reason == "incident replay window"
    ));
}

#[test]
fn cli_parses_lake_spark_maintenance_template_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "spark-template",
        "maintenance",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--table",
        "public.sales",
        "--target-table",
        "retail_sales__public__sales__scd2",
        "--epoch-id",
        "epoch-1",
        "--accept-complete-with-gaps",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::SparkTemplate {
                command: LakeSparkTemplateCommand::Maintenance(LakeSparkTemplateArgs {
                    table: Some(table),
                    target_table: Some(target_table),
                    epoch_id,
                    accept_complete_with_gaps: true,
                    ..
                })
            }
        } if table == "public.sales"
            && target_table == "retail_sales__public__sales__scd2"
            && epoch_id == "epoch-1"
    ));
}

#[test]
fn cli_parses_lake_spark_dashboard_template_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "lake",
        "spark-template",
        "dashboard",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--table",
        "public.sales",
        "--epoch-id",
        "epoch-1",
        "--accept-complete-with-gaps",
        "--format",
        "json",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Lake {
            command: LakeCommand::SparkTemplate {
                command: LakeSparkTemplateCommand::Dashboard(LakeSparkTemplateArgs {
                    table: Some(table),
                    epoch_id,
                    accept_complete_with_gaps: true,
                    format: QuickstartOutputFormat::Json,
                    ..
                })
            }
        } if table == "public.sales" && epoch_id == "epoch-1"
    ));
}
