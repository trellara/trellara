use super::support::*;
use trellara_protocol::RelationId;
use trellara_verify::{inspect_postgres_relation, PostgresRelationInspectionConfig};

#[tokio::test]
async fn postgres_relation_inspection_reports_columns_and_missing_tables() -> TestResult<()> {
    let Some(database_url) = integration_database_url(SOURCE_DATABASE_URL_ENV) else {
        return Ok(());
    };

    let sales = inspect_postgres_relation(PostgresRelationInspectionConfig {
        database_url: database_url.clone(),
        relation: RelationId::new(0, "public", "sales"),
    })
    .await?;
    let missing = inspect_postgres_relation(PostgresRelationInspectionConfig {
        database_url,
        relation: RelationId::new(0, "public", "trellara_missing_relation"),
    })
    .await?;

    assert!(sales.exists);
    assert!(sales.columns.iter().any(|column| column.name == "id"));
    assert!(!missing.exists);
    assert!(missing.columns.is_empty());
    Ok(())
}
