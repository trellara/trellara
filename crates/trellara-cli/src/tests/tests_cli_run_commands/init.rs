use super::*;

#[test]
fn cli_parses_init_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "init",
        "--source-database-url",
        "postgresql://source/app",
        "--target-database-url",
        "postgresql://target/app",
        "--source-id",
        "store-fleet",
        "--dataset-id",
        "sales",
        "--table",
        "public.sales",
        "--output",
        "trellara.yml",
        "--evaluate",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Init(InitArgs {
            source_id,
            dataset_id,
            table,
            evaluate: true,
            ..
        }) if source_id == "store-fleet" && dataset_id == "sales" && table == vec!["public.sales".to_string()]
    ));
}
