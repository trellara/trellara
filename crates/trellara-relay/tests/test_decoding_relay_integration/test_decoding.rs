use super::support::*;
use trellara_checkpoint::PostgresCheckpointStore;
use trellara_pg_capture::{PgCaptureConfig, TableSelector, TestDecodingCapture};
use trellara_relay::{load_source_checkpoint, Relay};

#[tokio::test]
async fn relay_publishes_test_decoding_transaction_then_records_checkpoint() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_source_state(&database_url).await?;
    let checkpoint_store = PostgresCheckpointStore::connect(&database_url, true).await?;
    reset_checkpoint(&database_url, &checkpoint_store).await?;

    let capture = TestDecodingCapture::connect(PgCaptureConfig {
        connection_uri: database_url.clone(),
        source_id: SOURCE_ID.to_string(),
        database_id: "retail".to_string(),
        dataset_id: DATASET_ID.to_string(),
        publication_name: "unused_relay_test_decoding_publication".to_string(),
        slot_name: SLOT_NAME.to_string(),
        tables: vec![TableSelector::new("public", "sales")],
        create_if_missing: true,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;
    let publisher = RecordingPublisher::succeeding();
    let mut relay = Relay::new(capture, publisher.clone(), checkpoint_store);

    write_source_transaction(&database_url).await?;

    let step = relay
        .run_once()
        .await?
        .expect("captured and published transaction");

    assert_eq!(step.envelope.source_id, SOURCE_ID);
    assert_eq!(step.envelope.dataset_id, DATASET_ID);
    assert_eq!(step.envelope.changes.len(), 1);
    assert_eq!(step.publish_acks.len(), 1);
    assert_eq!(publisher.published_messages().len(), 1);

    let checkpoint = load_source_checkpoint(
        &PostgresCheckpointStore::connect(&database_url, true).await?,
        SOURCE_ID,
        DATASET_ID,
    )
    .await?
    .expect("durable source checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, step.envelope.commit_lsn);
    assert_eq!(checkpoint.last_durable_lsn, step.envelope.commit_lsn);
    assert_eq!(checkpoint.last_applied_lsn, "");

    drop(relay);
    reset_source_state(&database_url).await?;
    Ok(())
}
