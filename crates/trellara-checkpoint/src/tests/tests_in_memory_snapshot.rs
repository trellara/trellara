use super::*;

#[tokio::test]
async fn in_memory_snapshot_run_transitions_preserve_recovery_evidence() {
    let store = InMemoryCheckpointStore::new();

    store
        .transition_snapshot_run(snapshot_run_for_test(SnapshotRunState::Planned, None, 0))
        .await
        .expect("planned run starts");
    store
        .transition_snapshot_run(snapshot_run_for_test(
            SnapshotRunState::SlotCreated,
            None,
            0,
        ))
        .await
        .expect("slot creation advances");
    store
        .transition_snapshot_run(snapshot_run_for_test(
            SnapshotRunState::SnapshotExported,
            Some("0/16B8000"),
            0,
        ))
        .await
        .expect("export records boundary");

    let loaded = store
        .load_snapshot_run(&FlowKey::new("source", "dataset"), "run")
        .await
        .expect("load run")
        .expect("run exists");

    assert_eq!(loaded.state, SnapshotRunState::SnapshotExported);
    assert_eq!(loaded.consistent_lsn.as_deref(), Some("0/16B8000"));
}

#[tokio::test]
async fn in_memory_snapshot_run_rejects_invalid_start_and_rewrites() {
    let store = InMemoryCheckpointStore::new();

    let error = store
        .transition_snapshot_run(snapshot_run_for_test(
            SnapshotRunState::CopyComplete,
            Some("0/16B8000"),
            10,
        ))
        .await
        .expect_err("run cannot start complete");
    assert!(error
        .to_string()
        .contains("snapshot run cannot start in copy_complete state"));

    store
        .transition_snapshot_run(snapshot_run_for_test(
            SnapshotRunState::SlotCreated,
            None,
            0,
        ))
        .await
        .expect("slot-created run can start");
    store
        .transition_snapshot_run(snapshot_run_for_test(
            SnapshotRunState::SnapshotExported,
            Some("0/16B8000"),
            0,
        ))
        .await
        .expect("snapshot export advances after slot creation");

    let error = store
        .transition_snapshot_run(snapshot_run_for_test(
            SnapshotRunState::CopyingTable,
            Some("0/16B9000"),
            10,
        ))
        .await
        .expect_err("consistent LSN rewrite rejected");
    assert!(error.to_string().contains("cannot change consistent LSN"));
}

#[tokio::test]
async fn in_memory_snapshot_table_progress_is_idempotent_and_sorted() {
    let store = InMemoryCheckpointStore::new();

    let orders = SnapshotTableProgress {
        relation: "public.orders".to_string(),
        ..snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, Some("0/16B8000"))
    };
    let sales = SnapshotTableProgress {
        relation: "public.sales".to_string(),
        ..snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, Some("0/16B8000"))
    };

    store
        .record_snapshot_table_progress(sales.clone())
        .await
        .expect("sales progress");
    store
        .record_snapshot_table_progress(orders.clone())
        .await
        .expect("orders progress");
    store
        .record_snapshot_table_progress(orders)
        .await
        .expect("idempotent replay");

    let listed = store
        .list_snapshot_table_progress(&FlowKey::new("source", "dataset"), "run")
        .await
        .expect("list progress");

    assert_eq!(
        listed
            .iter()
            .map(|progress| progress.relation.as_str())
            .collect::<Vec<_>>(),
        vec!["public.orders", "public.sales"]
    );
}

#[tokio::test]
async fn in_memory_snapshot_table_progress_rejects_recovery_regressions() {
    let store = InMemoryCheckpointStore::new();

    store
        .record_snapshot_table_progress(snapshot_table_progress_for_test(
            SnapshotRunState::CopyingTable,
            40,
            Some("0/16B8000"),
        ))
        .await
        .expect("current progress");

    let error = store
        .record_snapshot_table_progress(snapshot_table_progress_for_test(
            SnapshotRunState::CopyingTable,
            12,
            Some("0/16B8000"),
        ))
        .await
        .expect_err("copied row rollback rejected");
    assert!(error
        .to_string()
        .contains("cannot move copied rows backward"));

    let error = store
        .record_snapshot_table_progress(snapshot_table_progress_for_test(
            SnapshotRunState::CopyComplete,
            40,
            Some("0/16B9000"),
        ))
        .await
        .expect_err("watermark rewrite rejected");
    assert!(error.to_string().contains("cannot change watermark"));
}
