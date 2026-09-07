use super::*;

#[test]
fn converged_snapshots_match_even_when_row_order_differs() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-2", "999"), row("sale-1", "1299")],
    };
    let target = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299"), row("sale-2", "999")],
    };

    let comparison = compare_snapshots(source, target);

    assert!(comparison.is_converged());
    assert_eq!(comparison.source_row_count, 2);
    assert_eq!(comparison.target_row_count, 2);
}

#[test]
fn comparison_reports_missing_extra_and_mismatched_rows() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299"), row("sale-2", "999")],
    };
    let target = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1499"), row("sale-3", "500")],
    };

    let comparison = compare_snapshots(source, target);

    assert!(!comparison.is_converged());
    assert_eq!(comparison.missing_in_target, vec!["sale-2"]);
    assert_eq!(comparison.extra_in_target, vec!["sale-3"]);
    assert_eq!(comparison.mismatched_rows, vec!["sale-1"]);
}

#[test]
fn comparison_caps_drift_samples_but_reports_exact_counts() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![
            row("sale-1", "100"),
            row("sale-2", "200"),
            row("sale-3", "300"),
            row("sale-4", "400"),
            row("sale-5", "500"),
        ],
    };
    let target = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![
            row("sale-1", "101"),
            row("sale-6", "600"),
            row("sale-7", "700"),
            row("sale-8", "800"),
        ],
    };

    let comparison = compare_snapshots_with_sample_limit(source, target, 2);

    assert!(!comparison.is_converged());
    assert_eq!(comparison.missing_in_target_count, 4);
    assert_eq!(comparison.extra_in_target_count, 3);
    assert_eq!(comparison.mismatched_row_count, 1);
    assert_eq!(comparison.drift_sample_limit, 2);
    assert_eq!(comparison.missing_in_target, vec!["sale-2", "sale-3"]);
    assert_eq!(comparison.extra_in_target, vec!["sale-6", "sale-7"]);
    assert_eq!(comparison.mismatched_rows, vec!["sale-1"]);
}

#[test]
fn comparison_uses_counts_for_convergence_when_samples_are_empty() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "100"), row("sale-2", "200")],
    };
    let target = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-3", "300"), row("sale-4", "400")],
    };

    let comparison = compare_snapshots_with_sample_limit(source, target, 0);

    assert!(!comparison.is_converged());
    assert_eq!(comparison.missing_in_target_count, 2);
    assert_eq!(comparison.extra_in_target_count, 2);
    assert!(comparison.missing_in_target.is_empty());
    assert!(comparison.extra_in_target.is_empty());
}

#[test]
fn comparison_rejects_matching_rows_from_different_target_relation() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299")],
    };
    let target = TableSnapshot {
        relation: RelationId::new(84, "archive", "sales"),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299")],
    };

    let comparison = compare_snapshots(source, target);

    assert!(!comparison.is_converged());
    assert!(!comparison.relation_match);
    assert_eq!(comparison.relation.display_name(), "public.sales");
    assert_eq!(comparison.target_relation.display_name(), "archive.sales");
    assert_eq!(comparison.missing_in_target_count, 0);
    assert_eq!(comparison.extra_in_target_count, 0);
    assert_eq!(comparison.mismatched_row_count, 0);
}

#[test]
fn comparison_rejects_matching_rows_when_target_watermark_is_behind() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299")],
    };
    let target = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6B00".to_string(),
        rows: vec![row("sale-1", "1299")],
    };

    let comparison = compare_snapshots(source, target);

    assert!(!comparison.is_converged());
    assert!(!comparison.target_caught_up);
    assert_eq!(comparison.missing_in_target_count, 0);
    assert_eq!(comparison.extra_in_target_count, 0);
    assert_eq!(comparison.mismatched_row_count, 0);
}

#[test]
fn comparison_rejects_matching_rows_with_invalid_watermark_evidence() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "not-a-lsn".to_string(),
        rows: vec![row("sale-1", "1299")],
    };
    let target = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299")],
    };

    let comparison = compare_snapshots(source, target);

    assert!(!comparison.is_converged());
    assert!(!comparison.target_caught_up);
}

#[test]
fn comparison_includes_stable_evidence_digest() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-2", "999"), row("sale-1", "1299")],
    };
    let target = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299"), row("sale-2", "999")],
    };

    let comparison = compare_snapshots(source.clone(), target.clone());
    let reordered = compare_snapshots(source.canonical(), target.canonical());

    assert_eq!(comparison.evidence_sha256.len(), 64);
    assert_eq!(comparison.evidence_sha256, reordered.evidence_sha256);
}

#[test]
fn comparison_evidence_digest_changes_when_watermark_changes() {
    let source = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299")],
    };
    let mut target = TableSnapshot {
        relation: relation(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![row("sale-1", "1299")],
    };

    let first = compare_snapshots(source.clone(), target.clone());
    target.watermark_lsn = "0/16B6D28".to_string();
    let second = compare_snapshots(source, target);

    assert_ne!(first.evidence_sha256, second.evidence_sha256);
}
