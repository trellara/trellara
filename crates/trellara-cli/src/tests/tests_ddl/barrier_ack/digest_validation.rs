use super::*;

#[test]
fn ddl_barrier_ack_rejects_target_digest_evidence_without_statement_digests() {
    let mut args = base_args("target_postgres");
    args.plan_sha256 = Some("a".repeat(64));

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("missing statements");

    assert!(error
        .to_string()
        .contains("--plan-sha256 requires at least one --statement-sha256"));
}

#[test]
fn ddl_barrier_ack_rejects_malformed_target_plan_digest() {
    let mut args = base_args("target_postgres");
    args.plan_sha256 = Some("not-a-sha".to_string());
    args.statement_sha256s = vec!["b".repeat(64)];

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("bad digest");

    assert!(error
        .to_string()
        .contains("--plan-sha256 must be a 64-character SHA-256 hex digest"));
}

#[test]
fn ddl_barrier_ack_rejects_malformed_target_barrier_lsn() {
    let mut args = base_args("target_postgres");
    args.plan_sha256 = Some("a".repeat(64));
    args.statement_sha256s = vec!["b".repeat(64)];
    args.barrier_lsn = Some("not-a-lsn".to_string());

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("bad barrier lsn");

    assert!(error
        .to_string()
        .contains("--barrier-lsn not-a-lsn must be a non-zero PostgreSQL LSN"));
}
