use super::support::*;
use trellara_pg_capture::{PgCapture, PgCaptureConfig, TableSelector};
use trellara_protocol::RelationId;
use trellara_verify::{reseed_postgres_table, PostgresReseedConfig};

#[tokio::test]
async fn postgres_reseed_imports_exported_source_snapshot() -> TestResult<()> {
    let Some(source_url) = integration_database_url(SOURCE_DATABASE_URL_ENV) else {
        return Ok(());
    };
    let Some(target_url) = integration_database_url(TARGET_DATABASE_URL_ENV) else {
        return Ok(());
    };
    reset_reseed_table(&source_url, true).await?;
    reset_reseed_table(&target_url, false).await?;
    reset_slot(&source_url, EXPORTED_RESEED_SLOT).await?;
    insert_reseed_row(
        &source_url,
        "snapshot-visible-sale",
        "store-1",
        "1299",
        "source-user",
    )
    .await?;

    let exported = PgCapture::create_exported_logical_slot(PgCaptureConfig {
        connection_uri: source_url.clone(),
        source_id: "verify-source".to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "verify-reseed".to_string(),
        publication_name: "trellara_verify_exported_reseed_publication".to_string(),
        slot_name: EXPORTED_RESEED_SLOT.to_string(),
        tables: vec![TableSelector::new("public", RESEED_TABLE)],
        create_if_missing: true,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;

    insert_reseed_row(
        &source_url,
        "snapshot-hidden-sale",
        "store-2",
        "2400",
        "source-user",
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
        watermark_lsn: exported.consistent_lsn.clone(),
        source_snapshot_name: Some(exported.snapshot_name.clone()),
    })
    .await?;

    assert_eq!(summary.copied_rows, 1);
    assert_eq!(
        reseed_rows(&target_url).await?,
        vec![(
            "snapshot-visible-sale".to_string(),
            "store-1".to_string(),
            "1299".to_string(),
            "target-default".to_string(),
        )]
    );

    drop(exported);
    reset_slot(&source_url, EXPORTED_RESEED_SLOT).await?;
    reset_reseed_table(&source_url, true).await?;
    reset_reseed_table(&target_url, false).await?;
    Ok(())
}
