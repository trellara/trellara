use super::support::*;
use trellara_protocol::RelationId;
use trellara_verify::{snapshot_postgres_table, PostgresSnapshotConfig};

#[tokio::test]
async fn postgres_snapshot_hashes_rows_and_excludes_configured_columns() -> TestResult<()> {
    let Some(database_url) = integration_database_url(SOURCE_DATABASE_URL_ENV) else {
        return Ok(());
    };
    reset_rows(&database_url).await?;
    insert_row(&database_url, "verify-sale-1", "store-1", "1299").await?;
    insert_row(&database_url, "verify-sale-2", "store-2", "2400").await?;

    let snapshot = snapshot_postgres_table(PostgresSnapshotConfig {
        database_url: database_url.clone(),
        relation: RelationId::new(0, "public", "sales"),
        primary_key: "id".to_string(),
        excluded_columns: vec!["updated_at".to_string()],
        row_filter: None,
        watermark_lsn: "0/verify".to_string(),
    })
    .await?;

    assert_eq!(snapshot.watermark_lsn, "0/verify");
    let rows = snapshot
        .rows
        .iter()
        .filter(|row| row.primary_key.starts_with("verify-sale-"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        vec![
            expected_row("verify-sale-1", "store-1", "1299"),
            expected_row("verify-sale-2", "store-2", "2400"),
        ]
    );

    let filtered = snapshot_postgres_table(PostgresSnapshotConfig {
        database_url: database_url.clone(),
        relation: RelationId::new(0, "public", "sales"),
        primary_key: "id".to_string(),
        excluded_columns: vec!["updated_at".to_string()],
        row_filter: Some("store_id = 'store-1'".to_string()),
        watermark_lsn: "0/verify".to_string(),
    })
    .await?;
    assert_eq!(
        filtered
            .rows
            .iter()
            .filter(|row| row.primary_key.starts_with("verify-sale-"))
            .cloned()
            .collect::<Vec<_>>(),
        vec![expected_row("verify-sale-1", "store-1", "1299")]
    );

    reset_rows(&database_url).await?;
    Ok(())
}
