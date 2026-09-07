use std::collections::HashMap;

use super::*;

#[test]
fn snapshot_handoff_requires_every_relation_complete_at_boundary() {
    let relations = vec!["public.sales".to_string(), "public.orders".to_string()];
    let progress = progress_map([
        progress_for_relation(
            "public.sales",
            SnapshotRunState::CopyComplete,
            Some("0/16B8000"),
        ),
        progress_for_relation(
            "public.orders",
            SnapshotRunState::CopyingTable,
            Some("0/16B8000"),
        ),
    ]);

    assert!(
        !snapshot_handoff_ready_at_boundary(input(&relations, &progress, "0/16B8000"))
            .expect("readiness check")
    );
    assert_eq!(
        snapshot_boundary_copied_rows(input(&relations, &progress, "0/16B8000"))
            .expect("copied rows"),
        40
    );
}

#[test]
fn snapshot_handoff_accepts_all_relations_at_same_boundary() {
    let relations = vec!["public.sales".to_string(), "public.orders".to_string()];
    let progress = progress_map([
        progress_for_relation(
            "public.sales",
            SnapshotRunState::CopyComplete,
            Some("0/16B8000"),
        ),
        progress_for_relation(
            "public.orders",
            SnapshotRunState::CopyComplete,
            Some("0/16B8000"),
        ),
    ]);

    assert!(
        snapshot_handoff_ready_at_boundary(input(&relations, &progress, "0/16B8000"))
            .expect("readiness check")
    );
    assert_eq!(
        snapshot_boundary_copied_rows(input(&relations, &progress, "0/16B8000"))
            .expect("copied rows"),
        80
    );
}

#[test]
fn snapshot_handoff_report_explains_missing_and_incomplete_tables() {
    let relations = vec![
        "public.sales".to_string(),
        "public.orders".to_string(),
        "public.customers".to_string(),
    ];
    let progress = progress_map([
        progress_for_relation(
            "public.sales",
            SnapshotRunState::CopyComplete,
            Some("0/16B8000"),
        ),
        progress_for_relation(
            "public.orders",
            SnapshotRunState::CopyingTable,
            Some("0/16B8000"),
        ),
    ]);

    let report = snapshot_handoff_readiness_report(input(&relations, &progress, "0/16B8000"))
        .expect("readiness report");

    assert!(!report.ready);
    assert_eq!(report.relation_count, 3);
    assert_eq!(report.copied_rows_at_boundary, 40);
    assert_blocker(&report.blockers, "table_not_copy_complete", "public.orders");
    assert_blocker(
        &report.blockers,
        "missing_table_progress",
        "public.customers",
    );
    assert_eq!(report.recovery_actions.len(), 2);
    assert!(report
        .recovery_actions
        .iter()
        .any(|action| action.contains("resume table copy")));
}

#[test]
fn snapshot_handoff_report_explains_watermark_failures() {
    let relations = vec!["public.sales".to_string(), "public.orders".to_string()];
    let progress = progress_map([
        progress_for_relation("public.sales", SnapshotRunState::CopyComplete, None),
        progress_for_relation(
            "public.orders",
            SnapshotRunState::CopyComplete,
            Some("0/16B9000"),
        ),
    ]);

    let report = snapshot_handoff_readiness_report(input(&relations, &progress, "0/16B8000"))
        .expect("readiness report");

    assert!(!report.ready);
    assert_eq!(report.copied_rows_at_boundary, 0);
    assert_blocker(&report.blockers, "missing_watermark_lsn", "public.sales");
    assert_blocker(&report.blockers, "watermark_lsn_mismatch", "public.orders");
    assert!(report
        .recovery_actions
        .iter()
        .any(|action| action.contains("fresh snapshot handoff")));
}

#[test]
fn snapshot_handoff_rejects_empty_relation_set() {
    let progress = progress_map([progress_for_relation(
        "public.sales",
        SnapshotRunState::CopyComplete,
        Some("0/16B8000"),
    )]);

    let error = snapshot_handoff_ready_at_boundary(input(&[], &progress, "0/16B8000"))
        .expect_err("empty relation set rejected");

    assert!(error.to_string().contains("at least one relation"));
}

#[test]
fn snapshot_handoff_rejects_invalid_consistent_lsn() {
    let relations = vec!["public.sales".to_string()];
    let progress = progress_map([progress_for_relation(
        "public.sales",
        SnapshotRunState::CopyComplete,
        Some("0/16B8000"),
    )]);

    for consistent_lsn in ["not-a-lsn", "0/0"] {
        let error =
            snapshot_handoff_ready_at_boundary(input(&relations, &progress, consistent_lsn))
                .expect_err("invalid consistent lsn rejected");

        assert!(error.to_string().contains("consistent_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}

#[test]
fn snapshot_handoff_rejects_mismatched_progress_identity() {
    let relations = vec!["public.sales".to_string()];
    for (field, mutate) in [
        ("source_id", mutate_source as fn(&mut SnapshotTableProgress)),
        ("dataset_id", mutate_dataset),
        ("run_id", mutate_run),
    ] {
        let mut progress = progress_for_relation(
            "public.sales",
            SnapshotRunState::CopyComplete,
            Some("0/16B8000"),
        );
        mutate(&mut progress);
        let progress = progress_map([progress]);

        let error = snapshot_handoff_ready_at_boundary(input(&relations, &progress, "0/16B8000"))
            .expect_err("identity mismatch rejected");

        assert!(error.to_string().contains(field));
    }

    let mut progress = progress_for_relation(
        "public.sales",
        SnapshotRunState::CopyComplete,
        Some("0/16B8000"),
    );
    mutate_relation(&mut progress);
    let progress = HashMap::from([("public.sales".to_string(), progress)]);
    let error = snapshot_handoff_ready_at_boundary(input(&relations, &progress, "0/16B8000"))
        .expect_err("relation mismatch rejected");

    assert!(error.to_string().contains("relation"));
}

fn input<'a>(
    relations: &'a [String],
    progress: &'a HashMap<String, SnapshotTableProgress>,
    consistent_lsn: &'a str,
) -> SnapshotHandoffReadinessInput<'a> {
    SnapshotHandoffReadinessInput {
        source_id: "source",
        dataset_id: "dataset",
        run_id: "run",
        consistent_lsn,
        relations,
        progress,
    }
}

fn progress_map(
    entries: impl IntoIterator<Item = SnapshotTableProgress>,
) -> HashMap<String, SnapshotTableProgress> {
    entries
        .into_iter()
        .map(|progress| (progress.relation.clone(), progress))
        .collect()
}

fn progress_for_relation(
    relation: &str,
    state: SnapshotRunState,
    watermark_lsn: Option<&str>,
) -> SnapshotTableProgress {
    SnapshotTableProgress {
        relation: relation.to_string(),
        copied_rows: 40,
        state,
        watermark_lsn: watermark_lsn.map(str::to_string),
        ..snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 0, None)
    }
}

fn mutate_source(progress: &mut SnapshotTableProgress) {
    progress.source_id = "source-b".to_string();
}

fn mutate_dataset(progress: &mut SnapshotTableProgress) {
    progress.dataset_id = "orders".to_string();
}

fn mutate_run(progress: &mut SnapshotTableProgress) {
    progress.run_id = "run-2".to_string();
}

fn mutate_relation(progress: &mut SnapshotTableProgress) {
    progress.relation = "public.orders".to_string();
}

fn assert_blocker(blockers: &[crate::SnapshotHandoffReadinessBlocker], code: &str, relation: &str) {
    assert!(
        blockers
            .iter()
            .any(|blocker| blocker.code == code && blocker.relation == relation),
        "missing blocker {code} for {relation}: {blockers:?}"
    );
}
