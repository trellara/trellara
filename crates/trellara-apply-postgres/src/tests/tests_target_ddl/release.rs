use super::*;
use trellara_checkpoint::{DdlBarrier, DdlBarrierAck, DdlBarrierSummary};

#[test]
fn target_ddl_release_decision_blocks_pending_required_sinks() {
    let summary = DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: release_boundary(),
            required_sinks: vec![
                "target_postgres".to_string(),
                "raw_cdc_lake".to_string(),
                "partition_visibility".to_string(),
            ],
            requires_global_partition_pause: true,
        },
        vec![target_ack()],
    )
    .expect("valid barrier summary");

    let decision = target_ddl_release_decision(&summary);

    assert!(!decision.release_dml);
    assert_eq!(decision.source_id, "source");
    assert_eq!(decision.database_id, "retail");
    assert_eq!(decision.dataset_id, "sales");
    assert_eq!(decision.barrier_lsn, "0/16B6C50");
    assert!(decision
        .cdc_transaction_boundary
        .contains("post-DDL DML stays invisible"));
    assert_eq!(decision.release_gate, "post_ddl_dml_release");
    assert_eq!(
        decision.blockers,
        vec!["pending required sink acknowledgements: partition_visibility, raw_cdc_lake"]
    );
    assert_eq!(
        decision.blocker_codes,
        vec!["partition_visibility_not_released", "pending_required_ack"]
    );
    assert_eq!(
        decision
            .blocker_details
            .iter()
            .map(|blocker| blocker.code.as_str())
            .collect::<Vec<_>>(),
        vec!["pending_required_ack", "partition_visibility_not_released"]
    );
    assert_eq!(
        decision.blocker_details[1].sinks,
        vec!["partition_visibility"]
    );
    assert!(decision
        .evidence
        .iter()
        .any(|line| line.contains("partition_visibility_watermark satisfied=false")));
    assert!(matches!(
        decision.require_released(),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers == vec![
                    "pending required sink acknowledgements: partition_visibility, raw_cdc_lake"
                ]
    ));
}

#[test]
fn target_ddl_release_decision_allows_post_ddl_dml_after_all_required_acks() {
    let summary = DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: release_boundary(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        vec![target_ack()],
    )
    .expect("valid barrier summary");

    let decision = target_ddl_release_decision(&summary);

    assert!(decision.release_dml);
    assert!(decision.blockers.is_empty());
    assert!(decision.blocker_codes.is_empty());
    assert!(decision.blocker_details.is_empty());
    decision.require_released().expect("release accepted");
    decision
        .require_released_at_boundary("00000000/016B6C50")
        .expect("canonical source commit boundary accepted");
}

#[test]
fn target_ddl_release_decision_rejects_wrong_post_ddl_dml_boundary() {
    let summary = DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: release_boundary(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        vec![target_ack()],
    )
    .expect("valid barrier summary");

    let decision = target_ddl_release_decision(&summary);
    let error = decision
        .require_released_at_boundary("0/16B6D00")
        .expect_err("wrong DML source boundary");

    assert!(matches!(
        error,
        ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers }
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("does not match DDL barrier_lsn 0/16B6C50")
    ));
}

#[test]
fn target_ddl_release_decision_rejects_malformed_post_ddl_dml_boundary() {
    let decision = target_ddl_release_decision(&released_barrier_summary());

    for source_commit_lsn in ["not-a-lsn", "0/0"] {
        let error = decision
            .require_released_at_boundary(source_commit_lsn)
            .expect_err("malformed DML source boundary");

        assert!(matches!(
            error,
            ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers }
                if barrier_id == "ddl-barrier-123"
                    && blockers[0].contains("source_commit_lsn")
                    && blockers[0].contains("must be a non-zero PostgreSQL LSN")
        ));
    }
}

#[test]
fn target_ddl_release_decision_requires_canonical_propagation_boundary() {
    let mut summary = released_barrier_summary();
    summary.cdc_transaction_boundary =
        "source commit LSN is the DDL barrier; post-DDL DML stays invisible".to_string();
    let decision = target_ddl_release_decision(&summary);

    let error = decision
        .require_released_at_boundary("0/16B6C50")
        .expect_err("missing canonical propagation boundary");

    assert!(matches!(
        error,
        ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers }
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("canonical propagation boundary")
                && blockers[0].contains(trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY)
    ));
}

#[test]
fn target_ddl_release_decision_rejects_spoofed_propagation_boundary_text() {
    let mut summary = released_barrier_summary();
    summary.cdc_transaction_boundary = format!(
        "source commit LSN is the DDL barrier; propagation_boundary=legacy_boundary; note={}; post-DDL DML stays invisible",
        trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY
    );
    let decision = target_ddl_release_decision(&summary);

    let error = decision
        .require_released()
        .expect_err("spoofed boundary text");

    assert!(matches!(
        error,
        ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers }
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("canonical propagation boundary")
    ));
}

#[test]
fn target_ddl_release_decision_surfaces_sink_rejection_codes() {
    let summary = DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: release_boundary(),
            required_sinks: vec!["target_postgres".to_string(), "raw_cdc_lake".to_string()],
            requires_global_partition_pause: false,
        },
        vec![target_ack(), stale_lake_ack()],
    )
    .expect("valid barrier summary");

    let decision = target_ddl_release_decision(&summary);

    assert!(!decision.release_dml);
    assert_eq!(decision.blocker_codes, vec!["ack_lsn_before_barrier"]);
    assert_eq!(decision.blocker_details.len(), 1);
    assert_eq!(decision.blocker_details[0].code, "rejected_or_stale_ack");
    assert_eq!(decision.blocker_details[0].sinks, vec!["raw_cdc_lake"]);
    assert!(decision
        .blockers
        .contains(&"rejected or stale sink acknowledgements: raw_cdc_lake".to_string()));
}

fn target_ack() -> DdlBarrierAck {
    DdlBarrierAck {
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
    }
}

fn stale_lake_ack() -> DdlBarrierAck {
    DdlBarrierAck {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        sink: "raw_cdc_lake".to_string(),
        ack_lsn: "0/16B6B00".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: "raw CDC lake metadata is stale".to_string(),
    }
}

fn release_boundary() -> String {
    format!(
        "source commit LSN is the DDL barrier; propagation_boundary={}; propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1; propagation_policy_sha256={}; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
        trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY,
        "a".repeat(64)
    )
}

fn released_barrier_summary() -> DdlBarrierSummary {
    DdlBarrierSummary::try_from_barrier_and_acks(
        DdlBarrier {
            source_id: "source".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            barrier_lsn: "0/16B6C50".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: release_boundary(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        vec![target_ack()],
    )
    .expect("released barrier summary")
}
