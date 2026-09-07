use super::*;

#[test]
fn completed_snapshot_summary_reuses_handoff_ready_run() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let plan = SnapshotCopyPlan::from_tables(&config.dataset.tables, None).expect("snapshot plan");
    let run = handoff_ready_run(&config, "snapshot-run-1");
    let mut progress = HashMap::new();
    progress.insert(
        "public.sales".to_string(),
        table_progress(
            &config,
            "snapshot-run-1",
            "public.sales",
            SnapshotRunState::CopyComplete,
            40,
            Some("0/16B8000"),
        ),
    );

    let summary =
        completed_snapshot_copy_summary(&config, "snapshot-run-1", &plan, Some(&run), &progress)
            .expect("completed summary");

    assert_eq!(summary.state, "stream_handoff_ready");
    assert_eq!(summary.skipped_table_count, 1);
    assert_eq!(summary.copied_rows, 40);
    assert!(summary.tables[0].skipped);
    assert_eq!(summary.consistent_lsn, "0/16B8000");
    assert_eq!(
        summary.handoff_proof_command.as_deref(),
        Some("trellara snapshot --config <config> --run-id snapshot-run-1 --table public.sales")
    );
}

#[test]
fn snapshot_copy_summary_keeps_partial_handoff_in_copying_state() {
    let config = TrellaraConfig::from_yaml(&multi_table_strict_yaml(), "test").expect("parse");

    let summary = snapshot_copy_summary(
        &config,
        SnapshotCopySummaryDraft {
            run_id: "snapshot-run-1".to_string(),
            slot: "trellara_snapshot_slot".to_string(),
            consistent_lsn: "0/16B8000".to_string(),
            handoff_ready: false,
            table_count: 1,
            skipped_table_count: 0,
            copied_rows: 40,
            tables: vec![SnapshotCopyTableSummary {
                relation: "public.sales".to_string(),
                state: SnapshotRunState::CopyComplete.to_string(),
                copied_rows: 40,
                skipped: false,
                watermark_lsn: "0/16B8000".to_string(),
            }],
            exported_snapshot_name: Some("00000003-0000001B-1".to_string()),
            handoff_blocker_codes: vec!["missing_table_progress".to_string()],
            recovery_actions: vec![
                "rerun or resume the snapshot until every selected table records copy_complete"
                    .to_string(),
            ],
        },
    );

    assert_eq!(summary.state, "copying_table");
    assert!(summary.consistency_note.contains("withheld stream handoff"));
    assert_eq!(
        summary.next_commands,
        vec!["trellara snapshot --config <config>".to_string()]
    );
    assert_eq!(summary.handoff_proof_command, None);
    assert_eq!(
        summary.handoff_blocker_codes,
        vec!["missing_table_progress".to_string()]
    );
    assert!(summary.recovery_actions[0].contains("resume the snapshot"));
}

#[test]
fn snapshot_handoff_recovery_surfaces_blocker_codes_from_progress() {
    let config = TrellaraConfig::from_yaml(&multi_table_strict_yaml(), "test").expect("parse");
    let relations = vec!["public.sales".to_string(), "public.payments".to_string()];
    let progress = HashMap::from([(
        "public.sales".to_string(),
        table_progress(
            &config,
            "snapshot-run-1",
            "public.sales",
            SnapshotRunState::CopyComplete,
            40,
            Some("0/16B9000"),
        ),
    )]);

    let (codes, actions) = snapshot_handoff_recovery_for_flow(
        &config.source.id,
        &config.dataset.id,
        "snapshot-run-1",
        &relations,
        &progress,
        "0/16B8000",
    );

    assert_eq!(
        codes,
        vec![
            "watermark_lsn_mismatch".to_string(),
            "missing_table_progress".to_string(),
        ]
    );
    assert!(actions
        .iter()
        .any(|action| action.contains("fresh snapshot handoff")));
    assert!(actions
        .iter()
        .any(|action| action.contains("every selected table")));
}

#[test]
fn completed_snapshot_summary_requires_terminal_run_and_complete_tables() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let plan = SnapshotCopyPlan::from_tables(&config.dataset.tables, None).expect("snapshot plan");
    let active_run = SnapshotRun {
        state: SnapshotRunState::CopyingTable,
        current_relation: Some("public.sales".to_string()),
        copied_rows: 0,
        ..handoff_ready_run(&config, "snapshot-run-1")
    };
    let mut incomplete_progress = HashMap::new();
    incomplete_progress.insert(
        "public.sales".to_string(),
        table_progress(
            &config,
            "snapshot-run-1",
            "public.sales",
            SnapshotRunState::CopyingTable,
            0,
            Some("0/16B8000"),
        ),
    );

    assert!(completed_snapshot_copy_summary(
        &config,
        "snapshot-run-1",
        &plan,
        Some(&active_run),
        &incomplete_progress,
    )
    .is_none());

    let ready_run = SnapshotRun {
        state: SnapshotRunState::StreamHandoffReady,
        current_relation: None,
        ..active_run
    };
    assert!(completed_snapshot_copy_summary(
        &config,
        "snapshot-run-1",
        &plan,
        Some(&ready_run),
        &incomplete_progress,
    )
    .is_none());
}

#[test]
fn completed_snapshot_summary_requires_full_handoff_boundary_at_one_lsn() {
    let config = TrellaraConfig::from_yaml(&multi_table_strict_yaml(), "test").expect("parse");
    let plan = SnapshotCopyPlan::from_tables(&config.dataset.tables, Some("public.sales"))
        .expect("snapshot plan");
    let run = handoff_ready_run(&config, "snapshot-run-1");
    let mut progress = HashMap::new();
    progress.insert(
        "public.sales".to_string(),
        table_progress(
            &config,
            "snapshot-run-1",
            "public.sales",
            SnapshotRunState::CopyComplete,
            40,
            Some("0/16B8000"),
        ),
    );

    assert!(completed_snapshot_copy_summary(
        &config,
        "snapshot-run-1",
        &plan,
        Some(&run),
        &progress,
    )
    .is_none());

    progress.insert(
        "public.payments".to_string(),
        table_progress(
            &config,
            "snapshot-run-1",
            "public.payments",
            SnapshotRunState::CopyComplete,
            8,
            Some("0/16B7000"),
        ),
    );
    assert!(completed_snapshot_copy_summary(
        &config,
        "snapshot-run-1",
        &plan,
        Some(&run),
        &progress,
    )
    .is_none());

    progress
        .get_mut("public.payments")
        .expect("payments progress")
        .watermark_lsn = Some("0/16B8000".to_string());
    let summary =
        completed_snapshot_copy_summary(&config, "snapshot-run-1", &plan, Some(&run), &progress)
            .expect("completed selected-table summary");

    assert_eq!(summary.table_count, 1);
    assert_eq!(summary.skipped_table_count, 1);
    assert_eq!(summary.tables[0].relation, "public.sales");
    assert_eq!(summary.consistent_lsn, "0/16B8000");
}

#[test]
fn completed_snapshot_summary_rejects_stale_progress_identity() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let plan = SnapshotCopyPlan::from_tables(&config.dataset.tables, None).expect("snapshot plan");
    let run = handoff_ready_run(&config, "snapshot-run-1");
    let mut stale_progress = table_progress(
        &config,
        "snapshot-run-2",
        "public.sales",
        SnapshotRunState::CopyComplete,
        40,
        Some("0/16B8000"),
    );
    stale_progress.source_id = "other-source".to_string();
    let progress = HashMap::from([("public.sales".to_string(), stale_progress)]);

    assert!(completed_snapshot_copy_summary(
        &config,
        "snapshot-run-1",
        &plan,
        Some(&run),
        &progress,
    )
    .is_none());
}
