use super::support::*;
use trellara_protocol::RelationId;
use trellara_verify::{reseed_postgres_table, PostgresReseedConfig};

#[tokio::test]
async fn postgres_reseed_replaces_target_table_from_source_snapshot() -> TestResult<()> {
    let Some(source_url) = integration_database_url(SOURCE_DATABASE_URL_ENV) else {
        return Ok(());
    };
    let Some(target_url) = integration_database_url(TARGET_DATABASE_URL_ENV) else {
        return Ok(());
    };
    reset_reseed_table(&source_url, true).await?;
    reset_reseed_table(&target_url, false).await?;
    insert_reseed_row(
        &source_url,
        "reseed-sale-1",
        "store-1",
        "1299",
        "source-user",
    )
    .await?;
    insert_reseed_row(
        &source_url,
        "reseed-sale-2",
        "store-2",
        "2400",
        "source-user",
    )
    .await?;
    insert_reseed_row(
        &target_url,
        "stale-target-sale",
        "store-old",
        "1",
        "manager",
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

    assert_eq!(summary.copied_rows, 2);
    assert_eq!(
        summary.copied_columns,
        vec!["id", "store_id", "amount_cents", "receipt_bytes"]
    );
    assert_eq!(
        reseed_rows(&target_url).await?,
        vec![
            (
                "reseed-sale-1".to_string(),
                "store-1".to_string(),
                "1299".to_string(),
                "target-default".to_string(),
            ),
            (
                "reseed-sale-2".to_string(),
                "store-2".to_string(),
                "2400".to_string(),
                "target-default".to_string(),
            ),
        ]
    );

    reset_reseed_table(&source_url, true).await?;
    reset_reseed_table(&target_url, false).await?;
    Ok(())
}
