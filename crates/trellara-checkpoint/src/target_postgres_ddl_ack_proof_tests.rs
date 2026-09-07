use super::*;

#[test]
fn legacy_target_postgres_ddl_ack_detail_without_digests_is_invalid() {
    let detail = target_postgres_ddl_ack_detail(2);

    assert_eq!(
        detail,
        "target Postgres applied 2 DDL statements; release_gate=post_ddl_dml_release"
    );
    assert!(!target_postgres_ddl_ack_detail_is_valid(&detail));
}

#[test]
fn target_postgres_ddl_ack_detail_accepts_plan_and_statement_digests() {
    let digest = "a".repeat(64);
    let detail =
        target_postgres_ddl_ack_detail_with_digests(2, &digest, &[digest.clone(), "b".repeat(64)]);

    assert!(target_postgres_ddl_ack_detail_is_valid(&detail));
    assert!(detail.contains("plan_sha256="));
    assert!(detail.contains("statement_sha256="));
}

#[test]
fn target_postgres_ddl_ack_detail_accepts_cdc_barrier_lsn() {
    let digest = "a".repeat(64);
    let detail = target_postgres_ddl_ack_detail_with_boundary(
        1,
        &digest,
        std::slice::from_ref(&digest),
        Some("0/16B8000"),
    );

    assert!(target_postgres_ddl_ack_detail_is_valid(&detail));
    assert!(detail.contains("barrier_lsn=0/16B8000"));
}

#[test]
fn target_postgres_ddl_ack_detail_requires_matching_barrier_lsn_when_present() {
    let digest = "a".repeat(64);
    let detail = target_postgres_ddl_ack_detail_with_boundary(
        1,
        &digest,
        std::slice::from_ref(&digest),
        Some("0/16B8000"),
    );

    assert!(target_postgres_ddl_ack_detail_matches_barrier(
        &detail,
        crate::parse_lsn("0/16B8000")
    ));
    assert!(!target_postgres_ddl_ack_detail_matches_barrier(
        &detail,
        crate::parse_lsn("0/16C8000")
    ));
}

#[test]
fn target_postgres_ddl_ack_detail_rejects_digest_count_mismatch() {
    let digest = "a".repeat(64);
    let detail =
        target_postgres_ddl_ack_detail_with_digests(2, &digest, std::slice::from_ref(&digest));

    assert!(!target_postgres_ddl_ack_detail_is_valid(&detail));
}

#[test]
fn target_postgres_ddl_ack_detail_rejects_bad_plan_digest() {
    let detail = target_postgres_ddl_ack_detail_with_digests(1, "not-sha", &["a".repeat(64)]);

    assert!(!target_postgres_ddl_ack_detail_is_valid(&detail));
}

#[test]
fn target_postgres_ddl_ack_detail_rejects_bad_barrier_lsn() {
    let digest = "a".repeat(64);
    let detail = target_postgres_ddl_ack_detail_with_boundary(
        1,
        &digest,
        std::slice::from_ref(&digest),
        Some("not-a-lsn"),
    );

    assert!(!target_postgres_ddl_ack_detail_is_valid(&detail));
}

#[test]
fn target_postgres_ddl_ack_detail_requires_positive_statement_count() {
    let detail = target_postgres_ddl_ack_detail(0);

    assert!(!target_postgres_ddl_ack_detail_is_valid(&detail));
}

#[test]
fn target_postgres_ddl_ack_detail_rejects_spoofed_release_gate_token() {
    assert!(!target_postgres_ddl_ack_detail_is_valid(
        "target Postgres applied 1 DDL statements; previous_release_gate=post_ddl_dml_release"
    ));
}
