use super::support::*;
use trellara_protocol::RelationId;
use trellara_verify::{reseed_postgres_table, PostgresReseedConfig};

#[tokio::test]
async fn postgres_reseed_round_trips_bytea_columns() -> TestResult<()> {
    let Some(source_url) = integration_database_url(SOURCE_DATABASE_URL_ENV) else {
        return Ok(());
    };
    let Some(target_url) = integration_database_url(TARGET_DATABASE_URL_ENV) else {
        return Ok(());
    };
    reset_reseed_table(&source_url, true).await?;
    reset_reseed_table(&target_url, false).await?;
    insert_reseed_row_with_bytes(
        &source_url,
        "bytea-sale-1",
        "store-1",
        "1299",
        "source-user",
        &[1, 2, 3, 255],
    )
    .await?;

    let summary = reseed_postgres_table(PostgresReseedConfig {
        source_database_url: source_url.clone(),
        target_database_url: target_url.clone(),
        relation: RelationId::new(0, "public", RESEED_TABLE),
        primary_key: "id".to_string(),
        excluded_columns: vec!["updated_at".to_string()],
        target_owned_columns: vec!["reviewed_by".to_string()],
        row_filter: None,
        watermark_lsn: "0/reseed".to_string(),
        source_snapshot_name: None,
    })
    .await?;

    assert_eq!(summary.copied_rows, 1);
    assert!(summary
        .copied_columns
        .contains(&"receipt_bytes".to_string()));
    assert_eq!(
        reseed_row_bytes(&target_url, "bytea-sale-1").await?,
        Some(vec![1, 2, 3, 255])
    );

    reset_reseed_table(&source_url, true).await?;
    reset_reseed_table(&target_url, false).await?;
    Ok(())
}
