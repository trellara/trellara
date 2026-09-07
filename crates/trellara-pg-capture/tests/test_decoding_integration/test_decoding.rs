use super::support::*;
use trellara_pg_capture::{ChangeSource, PgCaptureConfig, TableSelector, TestDecodingCapture};
use trellara_protocol::Operation;

#[tokio::test]
async fn test_decoding_capture_emits_committed_transaction() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_slot_and_row(&database_url).await?;

    let mut capture = TestDecodingCapture::connect(PgCaptureConfig {
        connection_uri: database_url.clone(),
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
        publication_name: "unused_test_decoding_publication".to_string(),
        slot_name: SLOT_NAME.to_string(),
        tables: vec![TableSelector::new("public", "sales")],
        create_if_missing: true,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;

    write_source_transaction(&database_url).await?;

    let envelope = capture
        .next_transaction()
        .await?
        .expect("captured transaction");

    assert_eq!(envelope.source_id, "source-integration");
    assert_eq!(envelope.dataset_id, "retail-sales");
    assert_eq!(envelope.changes.len(), 1);
    assert_eq!(envelope.changes[0].operation, Operation::Insert as i32);
    assert_eq!(
        envelope.changes[0].idempotency_key,
        format!(
            "source-integration:{}:{}:1",
            envelope.commit_lsn, envelope.transaction_id
        )
    );
    envelope.verify_checksum()?;

    drop(capture);
    reset_slot_and_row(&database_url).await?;
    Ok(())
}

#[tokio::test]
async fn test_decoding_capture_emits_truncate_transaction() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_slot(&database_url, TRUNCATE_SLOT_NAME).await?;
    reset_truncate_table(&database_url).await?;

    let mut capture = TestDecodingCapture::connect(PgCaptureConfig {
        connection_uri: database_url.clone(),
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
        publication_name: "unused_test_decoding_publication".to_string(),
        slot_name: TRUNCATE_SLOT_NAME.to_string(),
        tables: vec![TableSelector::new("public", TRUNCATE_TABLE_NAME)],
        create_if_missing: true,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;

    write_truncate_transaction(&database_url).await?;

    let envelope = capture
        .next_transaction()
        .await?
        .expect("captured transaction");

    assert_eq!(envelope.changes.len(), 1);
    assert_eq!(envelope.changes[0].operation, Operation::Truncate as i32);
    assert_eq!(
        envelope.changes[0]
            .relation
            .as_ref()
            .expect("relation")
            .table,
        TRUNCATE_TABLE_NAME
    );
    assert!(envelope.changes[0].before.is_none());
    assert!(envelope.changes[0].after.is_none());
    envelope.verify_checksum()?;

    drop(capture);
    reset_slot(&database_url, TRUNCATE_SLOT_NAME).await?;
    reset_truncate_table(&database_url).await?;
    Ok(())
}

#[tokio::test]
async fn test_decoding_slot_status_reports_retained_wal() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_slot(&database_url, TRUNCATE_SLOT_NAME).await?;
    reset_truncate_table(&database_url).await?;

    let capture = TestDecodingCapture::connect(PgCaptureConfig {
        connection_uri: database_url.clone(),
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
        publication_name: "unused_test_decoding_publication".to_string(),
        slot_name: TRUNCATE_SLOT_NAME.to_string(),
        tables: vec![TableSelector::new("public", TRUNCATE_TABLE_NAME)],
        create_if_missing: true,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;

    let status = capture.inspect_slot_status().await?;

    assert_eq!(status.slot_name, TRUNCATE_SLOT_NAME);
    assert!(status.exists);
    assert_eq!(status.plugin, Some("test_decoding".to_string()));
    assert_eq!(status.expected_plugin, "test_decoding");
    assert!(status.restart_lsn.is_some());
    assert!(status.retained_wal_bytes.unwrap_or_default() >= 0);
    assert!(status.issues.is_empty());

    drop(capture);
    reset_slot(&database_url, TRUNCATE_SLOT_NAME).await?;
    reset_truncate_table(&database_url).await?;
    Ok(())
}
