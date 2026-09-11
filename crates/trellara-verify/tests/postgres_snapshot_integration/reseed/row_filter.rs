use super::support::*;
use trellara_protocol::RelationId;
use trellara_verify::{reseed_postgres_table, PostgresReseedConfig};

#[tokio::test]
async fn postgres_reseed_with_row_filter_replaces_only_matching_target_rows() -> TestResult<()> {
    let Some(source_url) = integration_database_url(SOURCE_DATABASE_URL_ENV) else {
        return Ok(());
    };
    let Some(target_url) = integration_database_url(TARGET_DATABASE_URL_ENV) else {
        return Ok(());
    };
    let _guard = RESEED_INTEGRATION_LOCK.lock().await;
    reset_reseed_table(&source_url, true).await?;
    reset_reseed_table(&target_url, false).await?;
    insert_reseed_row(
        &source_url,
        "filtered-sale-1",
        "store-1",
        "1299",
        "source-user",
    )
    .await?;
    insert_reseed_row(
        &source_url,
        "filtered-sale-2",
        "store-2",
        "2400",
        "source-user",
    )
    .await?;
    insert_reseed_row(&target_url, "stale-sale", "store-1", "1", "manager").await?;
    insert_reseed_row(&target_url, "preserved-sale", "store-2", "777", "manager").await?;

    let summary = reseed_postgres_table(PostgresReseedConfig {
        source_database_url: source_url.clone(),
        target_database_url: target_url.clone(),
        relation: RelationId::new(0, "public", RESEED_TABLE),
        primary_key: "id".to_string(),
        excluded_columns: vec!["updated_at".to_string()],
        target_owned_columns: vec!["reviewed_by".to_string()],
        row_filter: Some("store_id = 'store-1'".to_string()),
        watermark_lsn: "0/reseed".to_string(),
        source_snapshot_name: None,
    })
    .await?;

    assert_eq!(summary.copied_rows, 1);
    assert_eq!(
        reseed_rows(&target_url).await?,
        vec![
            (
                "filtered-sale-1".to_string(),
                "store-1".to_string(),
                "1299".to_string(),
                "target-default".to_string(),
            ),
            (
                "preserved-sale".to_string(),
                "store-2".to_string(),
                "777".to_string(),
                "manager".to_string(),
            ),
        ]
    );

    reset_reseed_table(&source_url, true).await?;
    reset_reseed_table(&target_url, false).await?;
    Ok(())
}
