use super::support::*;
use trellara_pg_capture::{PgCapture, PgCaptureConfig, TableSelector};

#[tokio::test]
async fn pgoutput_bootstrap_reports_consistent_slot_lsn() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_publication(&database_url, PGOUTPUT_PUBLICATION_NAME).await?;
    reset_slot(&database_url, PGOUTPUT_SLOT_NAME).await?;

    let bootstrap = PgCapture::bootstrap(PgCaptureConfig {
        connection_uri: database_url.clone(),
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
        publication_name: PGOUTPUT_PUBLICATION_NAME.to_string(),
        slot_name: PGOUTPUT_SLOT_NAME.to_string(),
        tables: vec![TableSelector::new("public", "sales")],
        create_if_missing: true,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;

    assert_eq!(bootstrap.publication_name, PGOUTPUT_PUBLICATION_NAME);
    assert_eq!(bootstrap.slot_name, PGOUTPUT_SLOT_NAME);
    assert!(bootstrap
        .consistent_lsn
        .as_deref()
        .is_some_and(|lsn| lsn.contains('/')));
    assert_eq!(bootstrap.exported_snapshot_name, None);
    assert_eq!(bootstrap.relations.len(), 1);
    assert!(bootstrap
        .preflight
        .iter()
        .all(|table| table.issues.is_empty()));

    reset_publication(&database_url, PGOUTPUT_PUBLICATION_NAME).await?;
    reset_slot(&database_url, PGOUTPUT_SLOT_NAME).await?;
    Ok(())
}

#[tokio::test]
async fn replication_protocol_slot_exports_importable_snapshot() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_slot(&database_url, EXPORTED_SNAPSHOT_SLOT_NAME).await?;

    let exported = PgCapture::create_exported_logical_slot(PgCaptureConfig {
        connection_uri: database_url.clone(),
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
        publication_name: PGOUTPUT_PUBLICATION_NAME.to_string(),
        slot_name: EXPORTED_SNAPSHOT_SLOT_NAME.to_string(),
        tables: vec![TableSelector::new("public", "sales")],
        create_if_missing: true,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;

    assert_eq!(exported.slot_name, EXPORTED_SNAPSHOT_SLOT_NAME);
    assert!(exported.consistent_lsn.contains('/'));
    assert!(!exported.snapshot_name.is_empty());
    assert_eq!(exported.output_plugin, "pgoutput");

    let client = connect(&database_url).await?;
    client
        .batch_execute(&format!(
            "begin isolation level repeatable read; set transaction snapshot '{}'; commit",
            exported.snapshot_name.replace('\'', "''")
        ))
        .await?;

    drop(exported);
    reset_slot(&database_url, EXPORTED_SNAPSHOT_SLOT_NAME).await?;
    Ok(())
}
