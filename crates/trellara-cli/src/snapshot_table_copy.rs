use trellara_checkpoint::{
    PostgresCheckpointStore, SnapshotHandoffEvent, SnapshotRunState, SnapshotTableProgress,
};

use crate::{u64_to_i64_count, Result, TrellaraConfig};

pub(crate) async fn record_snapshot_table_copy_started(
    target_store: &PostgresCheckpointStore,
    config: &TrellaraConfig,
    run_id: &str,
    relation_name: &str,
    consistent_lsn: &str,
) -> Result<()> {
    target_store
        .record_snapshot_table_progress(SnapshotTableProgress {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            run_id: run_id.to_string(),
            relation: relation_name.to_string(),
            state: SnapshotRunState::CopyingTable,
            copied_rows: 0,
            watermark_lsn: Some(consistent_lsn.to_string()),
            updated_at: String::new(),
        })
        .await?;
    Ok(())
}

pub(crate) async fn record_snapshot_table_copy_completed(
    target_store: &PostgresCheckpointStore,
    config: &TrellaraConfig,
    run_id: &str,
    relation_name: &str,
    consistent_lsn: &str,
    copied_rows: u64,
) -> Result<SnapshotTableProgress> {
    let copied_rows_i64 = u64_to_i64_count("snapshot copied_rows", copied_rows)?;
    let completed_progress = SnapshotTableProgress {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        run_id: run_id.to_string(),
        relation: relation_name.to_string(),
        state: SnapshotRunState::CopyComplete,
        copied_rows: copied_rows_i64,
        watermark_lsn: Some(consistent_lsn.to_string()),
        updated_at: String::new(),
    };
    target_store
        .record_snapshot_table_progress(completed_progress.clone())
        .await?;
    target_store
        .record_snapshot_handoff_event(SnapshotHandoffEvent {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            relation: relation_name.to_string(),
            watermark_lsn: consistent_lsn.to_string(),
            copied_rows: copied_rows_i64,
            completed_at: String::new(),
        })
        .await?;
    Ok(completed_progress)
}
