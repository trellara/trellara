use super::*;

#[test]
fn table_reseed_summary_maps_verify_summary() {
    let summary = TableReseedSummary::from_summary(
        trellara_verify::PostgresReseedSummary {
            relation: trellara_protocol::RelationId::new(0, "public", "sales"),
            watermark_lsn: "0/16B6C50".to_string(),
            copied_rows: 2,
            copied_columns: vec!["id".to_string(), "amount_cents".to_string()],
        },
        Some("store_id = 'store-1'".to_string()),
    );

    assert_eq!(summary.relation, "public.sales");
    assert_eq!(summary.row_filter.as_deref(), Some("store_id = 'store-1'"));
    assert_eq!(summary.watermark_lsn, "0/16B6C50");
    assert_eq!(summary.copied_rows, 2);
    assert_eq!(
        summary.copied_columns,
        vec!["id".to_string(), "amount_cents".to_string()]
    );
}

#[test]
fn table_verify_summary_maps_row_filter_scope() {
    let source = trellara_verify::TableSnapshot {
        relation: trellara_protocol::RelationId::new(0, "public", "sales"),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![trellara_verify::RowSnapshot::from_columns(
            "sale-1",
            &[("id", Some("sale-1"))],
        )],
    };
    let target = source.clone();

    let summary = TableVerifySummary::from_comparison(
        trellara_verify::compare_snapshots(source, target),
        Some("store_id = 'store-1'".to_string()),
    );

    assert_eq!(summary.relation, "public.sales");
    assert_eq!(summary.target_relation, "public.sales");
    assert!(summary.relation_match);
    assert_eq!(summary.row_filter.as_deref(), Some("store_id = 'store-1'"));
    assert!(summary.converged);
    assert_eq!(summary.checksum_status, ChecksumStatus::Match);
    assert_eq!(
        summary.recommended_action,
        "no action; table converged at the verified watermark"
    );
}

#[test]
fn table_verify_summary_rejects_wrong_target_relation() {
    let source = trellara_verify::TableSnapshot {
        relation: trellara_protocol::RelationId::new(0, "public", "sales"),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![trellara_verify::RowSnapshot::from_columns(
            "sale-1",
            &[("id", Some("sale-1"))],
        )],
    };
    let target = trellara_verify::TableSnapshot {
        relation: trellara_protocol::RelationId::new(0, "archive", "sales"),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![trellara_verify::RowSnapshot::from_columns(
            "sale-1",
            &[("id", Some("sale-1"))],
        )],
    };

    let summary = TableVerifySummary::from_comparison(
        trellara_verify::compare_snapshots(source, target),
        None,
    );

    assert_eq!(summary.relation, "public.sales");
    assert_eq!(summary.target_relation, "archive.sales");
    assert!(!summary.relation_match);
    assert!(!summary.converged);
    assert_eq!(summary.checksum_status, ChecksumStatus::Mismatch);
    assert!(summary
        .recommended_action
        .contains("target relation archive.sales does not match source relation public.sales"));
}

#[test]
fn table_verify_summary_marks_checksum_mismatch_with_repair_guidance() {
    let relation = trellara_protocol::RelationId::new(0, "public", "sales");
    let source = trellara_verify::TableSnapshot {
        relation: relation.clone(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![trellara_verify::RowSnapshot::from_columns(
            "sale-1",
            &[("id", Some("sale-1")), ("amount_cents", Some("10"))],
        )],
    };
    let target = trellara_verify::TableSnapshot {
        relation,
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![trellara_verify::RowSnapshot::from_columns(
            "sale-1",
            &[("id", Some("sale-1")), ("amount_cents", Some("20"))],
        )],
    };

    let summary = TableVerifySummary::from_comparison(
        trellara_verify::compare_snapshots(source, target),
        None,
    );

    assert_eq!(summary.relation, "public.sales");
    assert!(!summary.converged);
    assert_eq!(summary.checksum_status, ChecksumStatus::Mismatch);
    assert_eq!(summary.missing_in_target_count, 0);
    assert_eq!(summary.extra_in_target_count, 0);
    assert_eq!(summary.mismatched_row_count, 1);
    assert_eq!(
        summary.drift_sample_limit,
        trellara_verify::DEFAULT_DRIFT_SAMPLE_LIMIT
    );
    assert_eq!(summary.mismatched_rows, vec!["sale-1".to_string()]);
    assert_eq!(
            summary.recommended_action,
            "repair public.sales: 1 rows have checksum mismatches; run trellara reseed --config <config> --table public.sales, then rerun trellara verify --config <config> --table public.sales"
        );
}

#[test]
fn table_verify_summary_recommends_reseed_with_row_presence_counts() {
    let relation = trellara_protocol::RelationId::new(0, "public", "sales");
    let source = trellara_verify::TableSnapshot {
        relation: relation.clone(),
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![
            trellara_verify::RowSnapshot::from_columns(
                "sale-1",
                &[("id", Some("sale-1")), ("amount_cents", Some("10"))],
            ),
            trellara_verify::RowSnapshot::from_columns(
                "sale-2",
                &[("id", Some("sale-2")), ("amount_cents", Some("20"))],
            ),
        ],
    };
    let target = trellara_verify::TableSnapshot {
        relation,
        watermark_lsn: "0/16B6C50".to_string(),
        rows: vec![
            trellara_verify::RowSnapshot::from_columns(
                "sale-1",
                &[("id", Some("sale-1")), ("amount_cents", Some("11"))],
            ),
            trellara_verify::RowSnapshot::from_columns(
                "sale-3",
                &[("id", Some("sale-3")), ("amount_cents", Some("30"))],
            ),
        ],
    };

    let summary = TableVerifySummary::from_comparison(
        trellara_verify::compare_snapshots(source, target),
        None,
    );

    assert_eq!(summary.missing_in_target_count, 1);
    assert_eq!(summary.extra_in_target_count, 1);
    assert_eq!(summary.mismatched_row_count, 1);
    assert_eq!(
            summary.recommended_action,
            "repair public.sales: 1 source rows are missing in target; 1 target rows are not present in source; 1 rows have checksum mismatches; run trellara reseed --config <config> --table public.sales, then rerun trellara verify --config <config> --table public.sales"
        );
}
