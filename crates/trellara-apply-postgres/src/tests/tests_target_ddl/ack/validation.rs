use super::super::*;

#[test]
fn target_ddl_ack_evidence_requires_post_ddl_release_gate() {
    let outcome = TargetDdlApplyOutcome {
        barrier_id: "ddl-barrier-123".to_string(),
        applied_statements: 1,
        release_gate: "manual_release".to_string(),
        plan_sha256: "a".repeat(64),
        statement_sha256s: vec!["b".repeat(64)],
    };

    assert!(matches!(
        target_ack_evidence(outcome, "source-a", "dataset-a", "0/16B9000", "schema-v2"),
        Err(ApplyError::MissingDdlField {
            field: "post_ddl_dml_release"
        })
    ));
}

#[test]
fn target_ddl_ack_evidence_requires_boundary_fields() {
    for (source_id, dataset_id, ack_lsn, schema_version, field) in [
        ("", "dataset-a", "0/16B9000", "schema-v2", "source_id"),
        ("source-a", "", "0/16B9000", "schema-v2", "dataset_id"),
        ("source-a", "dataset-a", "", "schema-v2", "ack_lsn"),
        ("source-a", "dataset-a", "0/16B9000", "", "schema_version"),
    ] {
        let outcome = valid_outcome();

        assert!(matches!(
            target_ack_evidence(outcome, source_id, dataset_id, ack_lsn, schema_version),
            Err(ApplyError::MissingDdlField { field: actual }) if actual == field
        ));
    }
}

#[test]
fn target_ddl_ack_evidence_rejects_malformed_ack_lsn() {
    for ack_lsn in [
        "not-a-lsn",
        "0/0",
        "xyz/16B9000",
        "100000000/0",
        "0/100000000",
    ] {
        assert!(matches!(
            target_ack_evidence(
                valid_outcome(),
                "source-a",
                "dataset-a",
                ack_lsn,
                "schema-v2"
            ),
            Err(ApplyError::InvalidDdlAckEvidence {
                field: "ack_lsn",
                ..
            })
        ));
    }
}

#[test]
fn target_ddl_ack_evidence_rejects_ack_lsn_before_barrier_lsn() {
    let error = valid_outcome()
        .target_ack_evidence_from_context(
            TargetDdlAckContext::new("source-a", "retail", "dataset-a", "0/16B6C50", "schema-v2")
                .with_barrier_lsn("0/16B8000"),
        )
        .expect_err("ACK before barrier");

    assert!(matches!(
        error,
        ApplyError::InvalidDdlAckEvidence {
            field: "ack_lsn",
            ..
        }
    ));
    assert!(error
        .to_string()
        .contains("must be at or beyond barrier_lsn 0/16B8000"));
}

#[test]
fn target_ddl_ack_evidence_accepts_ack_lsn_at_barrier_lsn() {
    let evidence = valid_outcome()
        .target_ack_evidence_from_context(
            TargetDdlAckContext::new(
                "source-a",
                "retail",
                "dataset-a",
                "00000000/016B8000",
                "schema-v2",
            )
            .with_barrier_lsn("0/16B8000"),
        )
        .expect("ACK at barrier");

    assert_eq!(evidence.ack_lsn, "0/16B8000");
    assert_eq!(evidence.barrier_lsn.as_deref(), Some("0/16B8000"));
}

#[test]
fn target_ddl_ack_evidence_records_canonical_barrier_lsn_when_ack_is_beyond_barrier() {
    let evidence = valid_outcome()
        .target_ack_evidence_from_context(
            TargetDdlAckContext::new("source-a", "retail", "dataset-a", "0/16B9000", "schema-v2")
                .with_barrier_lsn("00000000/016B8000"),
        )
        .expect("ACK beyond barrier");

    assert_eq!(evidence.ack_lsn, "0/16B9000");
    assert_eq!(evidence.barrier_lsn.as_deref(), Some("0/16B8000"));
    assert!(evidence
        .into_barrier_ack()
        .detail
        .contains("barrier_lsn=0/16B8000"));
}

#[test]
fn target_ddl_ack_evidence_rejects_schema_version_with_surrounding_whitespace() {
    let error = valid_outcome()
        .target_ack_evidence_for_database(
            "source-a",
            "retail",
            "dataset-a",
            "0/16B9000",
            " schema-v2 ",
        )
        .expect_err("spaced schema version");

    assert!(matches!(
        error,
        ApplyError::InvalidDdlAckEvidence {
            field: "schema_version",
            ..
        }
    ));
    assert!(error
        .to_string()
        .contains("must not contain surrounding whitespace"));
}

#[test]
fn target_ddl_ack_evidence_rejects_identity_with_surrounding_whitespace() {
    for (source_id, dataset_id, barrier_id, field) in [
        (" source-a ", "dataset-a", "ddl-barrier-123", "source_id"),
        ("source-a", " dataset-a ", "ddl-barrier-123", "dataset_id"),
        ("source-a", "dataset-a", " ddl-barrier-123 ", "barrier_id"),
    ] {
        let mut outcome = valid_outcome();
        outcome.barrier_id = barrier_id.to_string();

        let error = outcome
            .target_ack_evidence_for_database(
                source_id,
                "retail",
                dataset_id,
                "0/16B9000",
                "schema-v2",
            )
            .expect_err("spaced ACK identity");

        assert!(matches!(
            error,
            ApplyError::InvalidDdlAckEvidence {
                field: actual,
                ..
            } if actual == field
        ));
        assert!(error
            .to_string()
            .contains("must not contain surrounding whitespace"));
    }
}

#[test]
fn target_ddl_ack_evidence_rejects_zero_applied_statements() {
    let outcome = TargetDdlApplyOutcome {
        barrier_id: "ddl-barrier-123".to_string(),
        applied_statements: 0,
        release_gate: "post_ddl_dml_release".to_string(),
        plan_sha256: "a".repeat(64),
        statement_sha256s: Vec::new(),
    };

    let error = outcome
        .target_ack_evidence_for_database(
            "source-a",
            "retail",
            "dataset-a",
            "0/16B9000",
            "schema-v2",
        )
        .expect_err("zero-statement ACK");

    assert!(matches!(
        error,
        ApplyError::InvalidDdlAckEvidence {
            field: "applied_statements",
            ..
        }
    ));
    assert!(error
        .to_string()
        .contains("at least one applied DDL statement"));
}

#[test]
fn target_ddl_ack_evidence_requires_plan_digest() {
    let mut outcome = valid_outcome();
    outcome.plan_sha256 = "not-a-digest".to_string();

    assert!(matches!(
        target_ack_evidence(outcome, "source-a", "dataset-a", "0/16B9000", "schema-v2"),
        Err(ApplyError::InvalidDdlAckEvidence {
            field: "plan_sha256",
            ..
        })
    ));
}

#[test]
fn target_ddl_ack_evidence_requires_statement_digest_for_each_statement() {
    let mut outcome = valid_outcome();
    outcome.applied_statements = 2;

    assert!(matches!(
        target_ack_evidence(outcome, "source-a", "dataset-a", "0/16B9000", "schema-v2"),
        Err(ApplyError::InvalidDdlAckEvidence {
            field: "statement_sha256",
            ..
        })
    ));
}

fn target_ack_evidence(
    outcome: TargetDdlApplyOutcome,
    source_id: &str,
    dataset_id: &str,
    ack_lsn: &str,
    schema_version: &str,
) -> Result<TargetDdlAckEvidence> {
    outcome.target_ack_evidence_for_database(
        source_id,
        "retail",
        dataset_id,
        ack_lsn,
        schema_version,
    )
}
