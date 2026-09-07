use super::support::*;
use trellara_checkpoint::{ReseedEvent, SnapshotHandoffEvent, ValidationEvent};

#[tokio::test]
async fn postgres_store_persists_reseed_handoff_and_validation_events() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let store = connect_store(&database_url).await?;
    reset_store(&database_url).await?;
    let flow = flow_key();

    store
        .record_reseed_event(ReseedEvent {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            watermark_lsn: "0/16B8000".to_string(),
            table_count: 2,
            copied_rows: 42,
            completed_at: String::new(),
        })
        .await?;
    let reseed = store
        .load_latest_reseed_event(&flow)
        .await?
        .expect("latest reseed");
    assert_eq!(reseed.watermark_lsn, "0/16B8000");
    assert_eq!(reseed.table_count, 2);
    assert_eq!(reseed.copied_rows, 42);

    store
        .record_snapshot_handoff_event(SnapshotHandoffEvent {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            relation: "public.sales".to_string(),
            watermark_lsn: "0/16B8000".to_string(),
            copied_rows: 40,
            completed_at: String::new(),
        })
        .await?;
    let handoff = store
        .load_latest_snapshot_handoff_event(&flow)
        .await?
        .expect("latest snapshot handoff");
    assert_eq!(handoff.relation, "public.sales");
    assert_eq!(handoff.watermark_lsn, "0/16B8000");
    assert_eq!(handoff.copied_rows, 40);

    store
        .record_validation_event(ValidationEvent {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            source_watermark_lsn: "0/16B9000".to_string(),
            target_watermark_lsn: "0/16B9000".to_string(),
            converged: false,
            table_count: 2,
            drift_count: 1,
            drift_relations: vec!["public.sales".to_string()],
            evidence_sha256: Some("a".repeat(64)),
            completed_at: String::new(),
        })
        .await?;
    let validation = store
        .load_latest_validation_event(&flow)
        .await?
        .expect("latest validation");
    assert_eq!(validation.source_watermark_lsn, "0/16B9000");
    assert_eq!(validation.target_watermark_lsn, "0/16B9000");
    assert!(!validation.converged);
    assert_eq!(validation.table_count, 2);
    assert_eq!(validation.drift_count, 1);
    assert_eq!(validation.drift_relations, vec!["public.sales"]);
    let expected_digest = "a".repeat(64);
    assert_eq!(
        validation.evidence_sha256.as_deref(),
        Some(expected_digest.as_str())
    );

    Ok(())
}
