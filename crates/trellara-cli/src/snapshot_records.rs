use trellara_checkpoint::{
    SnapshotHandoffEvent, SnapshotRun, SnapshotRunState, SnapshotTableProgress,
};

use crate::{BootstrapSummary, SnapshotRunDraft, TrellaraConfig};

pub(crate) fn snapshot_run_record(
    config: &TrellaraConfig,
    run_id: &str,
    draft: SnapshotRunDraft<'_>,
) -> SnapshotRun {
    SnapshotRun {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        run_id: run_id.to_string(),
        state: draft.state,
        slot_name: draft.slot_name.to_string(),
        consistent_lsn: draft.consistent_lsn,
        current_relation: draft.current_relation,
        copied_rows: draft.copied_rows,
        failure_reason: draft.failure_reason,
        started_at: String::new(),
        updated_at: String::new(),
    }
}

pub(crate) fn snapshot_exported_run_record(
    config: &TrellaraConfig,
    run_id: &str,
    bootstrap: &BootstrapSummary,
    consistent_lsn: &str,
    copied_rows: i64,
) -> Option<SnapshotRun> {
    bootstrap.exported_snapshot_name.as_ref()?;
    Some(snapshot_run_record(
        config,
        run_id,
        SnapshotRunDraft {
            state: SnapshotRunState::SnapshotExported,
            slot_name: &bootstrap.slot,
            consistent_lsn: Some(consistent_lsn.to_string()),
            current_relation: None,
            copied_rows,
            failure_reason: None,
        },
    ))
}

pub(crate) fn snapshot_copy_failure_records(
    config: &TrellaraConfig,
    run_id: &str,
    slot_name: &str,
    consistent_lsn: &str,
    relation_name: &str,
    copied_rows_before_failure: i64,
    failure_reason: String,
) -> (SnapshotRun, SnapshotTableProgress) {
    (
        snapshot_run_record(
            config,
            run_id,
            SnapshotRunDraft {
                state: SnapshotRunState::FailedRecoverable,
                slot_name,
                consistent_lsn: Some(consistent_lsn.to_string()),
                current_relation: Some(relation_name.to_string()),
                copied_rows: copied_rows_before_failure,
                failure_reason: Some(failure_reason),
            },
        ),
        SnapshotTableProgress {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            run_id: run_id.to_string(),
            relation: relation_name.to_string(),
            state: SnapshotRunState::FailedRecoverable,
            copied_rows: 0,
            watermark_lsn: Some(consistent_lsn.to_string()),
            updated_at: String::new(),
        },
    )
}

pub(crate) fn verified_snapshot_run(
    config: &TrellaraConfig,
    run: &SnapshotRun,
    handoff: Option<&SnapshotHandoffEvent>,
) -> Option<SnapshotRun> {
    let consistent_lsn = run.consistent_lsn.as_ref()?;
    let handoff = handoff?;
    let eligible = run.source_id == config.source.id
        && run.dataset_id == config.dataset.id
        && handoff.source_id == config.source.id
        && handoff.dataset_id == config.dataset.id
        && handoff.watermark_lsn == *consistent_lsn
        && matches!(
            run.state,
            SnapshotRunState::StreamHandoffReady
                | SnapshotRunState::Streaming
                | SnapshotRunState::Verified
        );
    eligible.then(|| {
        snapshot_run_record(
            config,
            &run.run_id,
            SnapshotRunDraft {
                state: SnapshotRunState::Verified,
                slot_name: &run.slot_name,
                consistent_lsn: run.consistent_lsn.clone(),
                current_relation: run.current_relation.clone(),
                copied_rows: run.copied_rows,
                failure_reason: None,
            },
        )
    })
}

pub(crate) fn default_snapshot_run_id() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("snapshot-{}-{millis}", std::process::id())
}
