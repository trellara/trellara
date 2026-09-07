use super::*;

pub(super) fn handoff_ready_run(config: &TrellaraConfig, run_id: &str) -> SnapshotRun {
    SnapshotRun {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        run_id: run_id.to_string(),
        state: SnapshotRunState::StreamHandoffReady,
        slot_name: "trellara_snapshot_slot".to_string(),
        consistent_lsn: Some("0/16B8000".to_string()),
        current_relation: None,
        copied_rows: 40,
        failure_reason: None,
        started_at: String::new(),
        updated_at: String::new(),
    }
}

pub(super) fn snapshot_handoff_event(
    config: &TrellaraConfig,
    watermark_lsn: &str,
) -> SnapshotHandoffEvent {
    SnapshotHandoffEvent {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        relation: "public.sales".to_string(),
        watermark_lsn: watermark_lsn.to_string(),
        copied_rows: 40,
        completed_at: String::new(),
    }
}

pub(super) fn table_progress(
    config: &TrellaraConfig,
    run_id: &str,
    relation: &str,
    state: SnapshotRunState,
    copied_rows: i64,
    watermark_lsn: Option<&str>,
) -> SnapshotTableProgress {
    SnapshotTableProgress {
        source_id: config.source.id.clone(),
        dataset_id: config.dataset.id.clone(),
        run_id: run_id.to_string(),
        relation: relation.to_string(),
        state,
        copied_rows,
        watermark_lsn: watermark_lsn.map(str::to_string),
        updated_at: String::new(),
    }
}

pub(super) fn verified_bootstrap(exported_snapshot_name: Option<&str>) -> BootstrapSummary {
    BootstrapSummary {
        publication: "trellara_publication".to_string(),
        slot: "trellara_slot".to_string(),
        consistent_lsn: Some("0/16B8000".to_string()),
        exported_snapshot_name: exported_snapshot_name.map(str::to_string),
        relation_count: 1,
        relations: vec!["public.sales".to_string()],
        preflight: PreflightSummary {
            passed: true,
            issue_count: 0,
            tables: Vec::new(),
        },
    }
}
