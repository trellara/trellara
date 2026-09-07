use std::collections::HashMap;

use trellara_checkpoint::{PostgresCheckpointStore, SnapshotRunState, SnapshotTableProgress};

use crate::{
    snapshot_boundary_copied_rows_for_flow, snapshot_handoff_ready_for_flow, snapshot_run_record,
    u64_to_i64_count, Result, SnapshotRunDraft, TrellaraConfig,
};

pub(crate) struct SnapshotCopyCompletion<'a> {
    pub(crate) config: &'a TrellaraConfig,
    pub(crate) run_id: &'a str,
    pub(crate) slot: &'a str,
    pub(crate) consistent_lsn: &'a str,
    pub(crate) handoff_relations: &'a [String],
    pub(crate) planned_progress: &'a HashMap<String, SnapshotTableProgress>,
    pub(crate) copied_rows: u64,
}

pub(crate) async fn complete_snapshot_copy_run(
    target_store: &PostgresCheckpointStore,
    completion: SnapshotCopyCompletion<'_>,
) -> Result<(bool, i64)> {
    let handoff_ready = snapshot_handoff_ready_for_flow(
        &completion.config.source.id,
        &completion.config.dataset.id,
        completion.run_id,
        completion.handoff_relations,
        completion.planned_progress,
        completion.consistent_lsn,
    );
    let final_run_copied_rows = if handoff_ready {
        snapshot_boundary_copied_rows_for_flow(
            &completion.config.source.id,
            &completion.config.dataset.id,
            completion.run_id,
            completion.handoff_relations,
            completion.planned_progress,
            completion.consistent_lsn,
        )
    } else {
        u64_to_i64_count("snapshot copied_rows", completion.copied_rows)?
    };
    if handoff_ready {
        target_store
            .transition_snapshot_run(snapshot_run_record(
                completion.config,
                completion.run_id,
                SnapshotRunDraft {
                    state: SnapshotRunState::CopyComplete,
                    slot_name: completion.slot,
                    consistent_lsn: Some(completion.consistent_lsn.to_string()),
                    current_relation: None,
                    copied_rows: final_run_copied_rows,
                    failure_reason: None,
                },
            ))
            .await?;
        target_store
            .transition_snapshot_run(snapshot_run_record(
                completion.config,
                completion.run_id,
                SnapshotRunDraft {
                    state: SnapshotRunState::StreamHandoffReady,
                    slot_name: completion.slot,
                    consistent_lsn: Some(completion.consistent_lsn.to_string()),
                    current_relation: None,
                    copied_rows: final_run_copied_rows,
                    failure_reason: None,
                },
            ))
            .await?;
    } else {
        target_store
            .transition_snapshot_run(snapshot_run_record(
                completion.config,
                completion.run_id,
                SnapshotRunDraft {
                    state: SnapshotRunState::CopyingTable,
                    slot_name: completion.slot,
                    consistent_lsn: Some(completion.consistent_lsn.to_string()),
                    current_relation: None,
                    copied_rows: final_run_copied_rows,
                    failure_reason: None,
                },
            ))
            .await?;
    }

    Ok((handoff_ready, final_run_copied_rows))
}
