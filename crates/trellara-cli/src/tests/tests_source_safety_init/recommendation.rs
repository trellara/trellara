use super::*;

#[test]
fn source_safety_init_recommendation_uses_inspected_tables_and_primary_key() {
    let args = SourceSafetyArgs {
        config: None,
        format: SourceSafetyOutputFormat::Text,
        output: None,
        database_url: Some("postgresql://source/app".to_string()),
        target_database_url: None,
        source_id: "store-fleet".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        publication: "trellara_sales".to_string(),
        slot: "trellara_sales_slot".to_string(),
        capture: "pgoutput".to_string(),
        wal_retention_warn_bytes: None,
        table: vec!["public.sales".to_string(), "public.payments".to_string()],
        write_init: None,
        force: false,
    };
    let recommendation = source_safety_init_recommendation(
        &args,
        &[
            preflight_table_named("public", "sales"),
            preflight_table_named("public", "payments"),
        ],
    )
    .expect("init recommendation");

    assert_eq!(recommendation.output, "trellara.yml");
    assert_eq!(recommendation.table_count, 2);
    assert_eq!(recommendation.primary_key, "id");
    assert!(recommendation.evaluation_ready);
    assert!(recommendation.command.contains("trellara init"));
    assert!(recommendation
        .command
        .contains("--source-database-url postgresql://source/app"));
    assert!(recommendation
        .command
        .contains("--target-database-url <target-postgres-url>"));
    assert!(recommendation.command.contains("--source-id store-fleet"));
    assert!(recommendation.command.contains("--database-id retail"));
    assert!(recommendation.command.contains("--dataset-id sales"));
    assert!(recommendation
        .command
        .contains("--publication trellara_sales"));
    assert!(recommendation
        .command
        .contains("--slot trellara_sales_slot"));
    assert!(recommendation.command.contains("--table public.sales"));
    assert!(recommendation.command.contains("--table public.payments"));
    assert!(recommendation.command.contains("--primary-key id"));
    assert!(recommendation.command.contains("--output trellara.yml"));
    assert!(recommendation.command.contains("--evaluate"));
    assert!(recommendation
        .note
        .contains("replace <target-postgres-url>"));
}
