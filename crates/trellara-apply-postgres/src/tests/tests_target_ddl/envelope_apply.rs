use super::*;
use async_trait::async_trait;
use trellara_checkpoint::{
    DdlBarrier, DdlBarrierAck, DdlBarrierSummary, DDL_BARRIER_CDC_TRANSACTION_BOUNDARY,
};

#[test]
fn dml_replay_envelope_strips_ddl_events_and_refreshes_checksum() {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-ddl".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: Vec::new(),
    });
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-ddl",
        1,
        RelationId::new(42, "public", "sales"),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let replay = envelope.dml_replay_after_ddl_barrier();

    assert!(replay.ddl_events.is_empty());
    replay.verify_checksum().expect("refreshed checksum");
}

#[test]
fn envelope_outcome_carries_post_ddl_dml_release_decision() {
    let barrier_summary = DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: ddl_barrier_cdc_boundary(),
            required_sinks: vec!["target_postgres".to_string(), "raw_cdc_lake".to_string()],
            requires_global_partition_pause: false,
        },
        vec![DdlBarrierAck {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: trellara_checkpoint::target_postgres_ddl_ack_detail_with_digests(
                1,
                &"a".repeat(64),
                &["b".repeat(64)],
            ),
        }],
    )
    .expect("valid barrier summary");
    let release_decision = target_ddl_release_decision(&barrier_summary);
    let outcome = TargetDdlEnvelopeApplyOutcome {
        target_ack: TargetDdlAckEvidence {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B6C50".to_string(),
            barrier_lsn: Some("0/16B6C50".to_string()),
            schema_version: "schema-v2".to_string(),
            applied_statements: 1,
            release_gate: "post_ddl_dml_release".to_string(),
            plan_sha256: "a".repeat(64),
            statement_sha256s: vec!["b".repeat(64)],
        },
        barrier_summary,
        release_decision,
        ddl_propagation_decisions:
            "propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1"
                .to_string(),
        ddl_target_ack_required: 1,
        ddl_propagation_policy_sha256: "c".repeat(64),
    };

    assert!(!outcome.release_decision.release_dml);
    assert_eq!(
        outcome.release_decision.blockers,
        vec!["pending required sink acknowledgements: raw_cdc_lake"]
    );
}

#[test]
fn ddl_envelope_propagation_proof_exposes_apply_outcome_evidence() {
    let proof = crate::ddl_envelope_apply::ddl_envelope_propagation_proof(&ddl_envelope())
        .expect("DDL propagation proof");

    assert_eq!(
        proof.decisions,
        "propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1"
    );
    assert_eq!(proof.target_ack_required, 1);
    assert_eq!(proof.policy_sha256.len(), 64);
}

#[test]
fn ddl_dml_replay_proof_records_ack_release_then_dml_boundary() {
    let summary = released_barrier_summary();
    let release_decision = target_ddl_release_decision(&summary);
    let dml = ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 2,
        commit_lsn: "00000000/016B6C50".to_string(),
    };

    let proof = crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
        &target_ack_evidence(),
        &release_decision,
        &dml,
    )
    .expect("DDL/DML replay proof");

    assert_eq!(proof.contract, TARGET_DDL_DML_REPLAY_PROOF_CONTRACT);
    assert_eq!(proof.database_id, "retail");
    assert_eq!(proof.barrier_id, "ddl-barrier-123");
    assert_eq!(proof.barrier_lsn, "0/16B6C50");
    assert_eq!(proof.target_ack_lsn, "0/16B6C50");
    assert_eq!(proof.dml_commit_lsn, "00000000/016B6C50");
    assert_eq!(proof.schema_version, "schema-v2");
    assert_eq!(proof.ddl_applied_statements, 1);
    assert_eq!(proof.dml_applied_changes, 2);
    assert_eq!(proof.dml_decision, "Applied");
    assert_eq!(proof.release_gate, "post_ddl_dml_release");
    assert_eq!(
        proof.target_transaction_boundary,
        TARGET_DDL_TRANSACTION_BOUNDARY
    );
    assert!(proof
        .cdc_transaction_boundary
        .starts_with(DDL_BARRIER_CDC_TRANSACTION_BOUNDARY));
    assert!(proof
        .cdc_transaction_boundary
        .contains(DDL_PROPAGATION_CDC_BOUNDARY));
    assert_eq!(
        proof.proof_steps,
        vec![
            "target_postgres_recorded_ddl_ack",
            "release_decision_allowed_post_ddl_dml",
            "dml_replay_applied_changes_positive",
            "dml_replay_commit_lsn_matches_ddl_barrier_lsn",
        ]
    );
}

#[test]
fn ddl_dml_replay_proof_rejects_zero_statement_target_ack() {
    let summary = released_barrier_summary();
    let release_decision = target_ddl_release_decision(&summary);
    let mut target_ack = target_ack_evidence();
    target_ack.applied_statements = 0;
    target_ack.statement_sha256s.clear();

    let error = crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
        &target_ack,
        &release_decision,
        &applied_dml_outcome(),
    )
    .expect_err("zero-statement target ACK");

    assert!(matches!(
        error,
        ApplyError::InvalidDdlAckEvidence {
            field: "applied_statements",
            ..
        }
    ));
}

#[test]
fn ddl_dml_replay_proof_rejects_statement_digest_count_mismatch() {
    let summary = released_barrier_summary();
    let release_decision = target_ddl_release_decision(&summary);
    let mut target_ack = target_ack_evidence();
    target_ack.applied_statements = 2;

    let error = crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
        &target_ack,
        &release_decision,
        &applied_dml_outcome(),
    )
    .expect_err("statement digest mismatch");

    assert!(matches!(
        error,
        ApplyError::InvalidDdlAckEvidence {
            field: "statement_sha256",
            ..
        }
    ));
}

#[test]
fn missing_target_ddl_ack_returns_typed_error() {
    let error =
        crate::ddl_envelope_apply::require_target_ddl_ack(None).expect_err("missing target ack");

    assert!(matches!(
        error,
        ApplyError::MissingDdlField {
            field: "target_ddl_ack"
        }
    ));
}

pub(super) fn released_barrier_summary() -> DdlBarrierSummary {
    DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: ddl_barrier_cdc_boundary(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        vec![target_ack()],
    )
    .expect("released barrier summary")
}

pub(super) fn blocked_barrier_summary() -> DdlBarrierSummary {
    DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: ddl_barrier_cdc_boundary(),
            required_sinks: vec!["target_postgres".to_string(), "raw_cdc_lake".to_string()],
            requires_global_partition_pause: false,
        },
        vec![target_ack()],
    )
    .expect("blocked barrier summary")
}

fn target_ack() -> DdlBarrierAck {
    target_ack_evidence().into_barrier_ack()
}

fn ddl_barrier_cdc_boundary() -> String {
    format!(
        "{DDL_BARRIER_CDC_TRANSACTION_BOUNDARY}; propagation_boundary={DDL_PROPAGATION_CDC_BOUNDARY}; propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1; propagation_policy_sha256={}; post-DDL DML stays invisible until required sink ACKs reach barrier_lsn",
        "c".repeat(64)
    )
}

pub(super) fn target_ack_evidence() -> TargetDdlAckEvidence {
    TargetDdlAckEvidence {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        sink: "target_postgres".to_string(),
        ack_lsn: "0/16B6C50".to_string(),
        barrier_lsn: Some("0/16B6C50".to_string()),
        schema_version: "schema-v2".to_string(),
        applied_statements: 1,
        release_gate: "post_ddl_dml_release".to_string(),
        plan_sha256: "a".repeat(64),
        statement_sha256s: vec!["b".repeat(64)],
    }
}

fn applied_dml_outcome() -> ApplyOutcome {
    ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 2,
        commit_lsn: "0/16B6C50".to_string(),
    }
}

#[tokio::test]
async fn missing_ddl_barrier_summary_returns_typed_error() {
    let store = MissingSummaryStore;
    let error = crate::ddl_envelope_apply::require_ddl_barrier_summary(
        &store,
        &DdlBarrierLookup::new("source", "retail", "sales"),
        "ddl-barrier",
    )
    .await
    .expect_err("missing summary");

    assert!(matches!(
        error,
        ApplyError::MissingDdlField {
            field: "ddl_barrier_summary"
        }
    ));
}

struct MissingSummaryStore;

#[async_trait]
impl DdlBarrierStore for MissingSummaryStore {
    async fn record_ddl_barrier(&self, _barrier: DdlBarrier) -> trellara_checkpoint::Result<()> {
        Ok(())
    }

    async fn record_ddl_barrier_ack(&self, _ack: DdlBarrierAck) -> trellara_checkpoint::Result<()> {
        Ok(())
    }

    async fn ddl_barrier_summary(
        &self,
        _flow: &DdlBarrierLookup,
        _barrier_id: &str,
    ) -> trellara_checkpoint::Result<Option<DdlBarrierSummary>> {
        Ok(None)
    }
}
