use super::support::*;
use trellara_checkpoint::{SnapshotRun, SnapshotRunState, SnapshotTableProgress};

#[tokio::test]
async fn postgres_store_persists_snapshot_run_and_table_progress_state() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let store = connect_store(&database_url).await?;
    reset_store(&database_url).await?;
    let flow = flow_key();

    store
        .transition_snapshot_run(SnapshotRun {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            run_id: "snapshot-run-1".to_string(),
            state: SnapshotRunState::SlotCreated,
            slot_name: "trellara_snapshot_slot".to_string(),
            consistent_lsn: None,
            current_relation: None,
            copied_rows: 0,
            failure_reason: None,
            started_at: String::new(),
            updated_at: String::new(),
        })
        .await?;
    store
        .transition_snapshot_run(SnapshotRun {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            run_id: "snapshot-run-1".to_string(),
            state: SnapshotRunState::CopyingTable,
            slot_name: "trellara_snapshot_slot".to_string(),
            consistent_lsn: Some("0/16B8000".to_string()),
            current_relation: Some("public.sales".to_string()),
            copied_rows: 40,
            failure_reason: None,
            started_at: String::new(),
            updated_at: String::new(),
        })
        .await?;
    assert!(store
        .transition_snapshot_run(SnapshotRun {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            run_id: "snapshot-run-1".to_string(),
            state: SnapshotRunState::Planned,
            slot_name: "trellara_snapshot_slot".to_string(),
            consistent_lsn: None,
            current_relation: None,
            copied_rows: 0,
            failure_reason: None,
            started_at: String::new(),
            updated_at: String::new(),
        })
        .await
        .is_err());
    let loaded_snapshot_run = store
        .load_snapshot_run(&flow, "snapshot-run-1")
        .await?
        .expect("loaded snapshot run");
    assert_eq!(loaded_snapshot_run.state, SnapshotRunState::CopyingTable);
    let snapshot_run = store
        .load_latest_snapshot_run(&flow)
        .await?
        .expect("latest snapshot run");
    assert_eq!(snapshot_run.run_id, "snapshot-run-1");
    assert_eq!(snapshot_run.state, SnapshotRunState::CopyingTable);
    assert_eq!(snapshot_run.consistent_lsn.as_deref(), Some("0/16B8000"));
    assert_eq!(
        snapshot_run.current_relation.as_deref(),
        Some("public.sales")
    );
    assert_eq!(snapshot_run.copied_rows, 40);

    store
        .record_snapshot_table_progress(SnapshotTableProgress {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            run_id: "snapshot-run-1".to_string(),
            relation: "public.sales".to_string(),
            state: SnapshotRunState::CopyingTable,
            copied_rows: 12,
            watermark_lsn: Some("0/16B8000".to_string()),
            updated_at: String::new(),
        })
        .await?;
    store
        .record_snapshot_table_progress(SnapshotTableProgress {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            run_id: "snapshot-run-1".to_string(),
            relation: "public.sales".to_string(),
            state: SnapshotRunState::CopyComplete,
            copied_rows: 40,
            watermark_lsn: Some("0/16B8000".to_string()),
            updated_at: String::new(),
        })
        .await?;
    let progress = store
        .list_snapshot_table_progress(&flow, "snapshot-run-1")
        .await?;
    assert_eq!(progress.len(), 1);
    assert_eq!(progress[0].relation, "public.sales");
    assert_eq!(progress[0].state, SnapshotRunState::CopyComplete);
    assert_eq!(progress[0].copied_rows, 40);
    assert_eq!(progress[0].watermark_lsn.as_deref(), Some("0/16B8000"));
    assert!(store
        .record_snapshot_table_progress(SnapshotTableProgress {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            run_id: "snapshot-run-1".to_string(),
            relation: "public.sales".to_string(),
            state: SnapshotRunState::CopyingTable,
            copied_rows: 12,
            watermark_lsn: Some("0/16B9000".to_string()),
            updated_at: String::new(),
        })
        .await
        .is_err());

    Ok(())
}
