use super::*;

#[tokio::test]
async fn ddl_barrier_summary_blocks_unexpected_sink_acks() {
    let store = InMemoryCheckpointStore::new();
    let flow = ddl_flow();
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: flow.source_id.clone(),
        database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-unexpected".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect("record barrier");

    for sink in ["target_postgres", "typo_target_postgres"] {
        let detail = if sink == "target_postgres" {
            target_postgres_ddl_ack_detail_with_digests(1, &"a".repeat(64), &["b".repeat(64)])
        } else {
            "accepted".to_string()
        };
        store
            .record_ddl_barrier_ack(DdlBarrierAck {
                source_id: flow.source_id.clone(),
                database_id: "retail".to_string(),
                dataset_id: flow.dataset_id.clone(),
                barrier_id: "ddl-barrier-unexpected".to_string(),
                sink: sink.to_string(),
                ack_lsn: "0/16B9000".to_string(),
                schema_version: "schema-v2".to_string(),
                accepted: true,
                detail,
            })
            .await
            .expect("record ack");
    }

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-unexpected")
        .await
        .expect("load summary")
        .expect("summary");

    assert!(!summary.release_dml);
    assert_eq!(summary.acked_sinks, vec!["target_postgres"]);
    assert_eq!(summary.unexpected_ack_count, 1);
    assert_eq!(summary.unexpected_sinks, vec!["typo_target_postgres"]);
    assert_eq!(
        summary.release_blockers,
        vec!["unexpected sink acknowledgements: typo_target_postgres"]
    );
    assert_eq!(summary.release_blocker_codes, vec!["unexpected_sink_ack"]);
    let unexpected = summary
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "typo_target_postgres")
        .expect("unexpected evidence");
    assert_eq!(unexpected.status, "unexpected");
    assert!(!unexpected.release_eligible);
    assert_eq!(
        unexpected.rejection_reason.as_deref(),
        Some("sink is not required for this barrier")
    );
    assert_eq!(
        unexpected.rejection_code.as_deref(),
        Some("unexpected_sink_ack")
    );
    assert_eq!(summary.release_actions.len(), 1);
    assert_eq!(
        summary.release_actions[0].code,
        "remove_unexpected_sink_ack"
    );
    assert_eq!(
        summary.release_actions[0].sinks,
        vec!["typo_target_postgres"]
    );
}

#[test]
fn ddl_barrier_summary_unexpected_duplicate_acks_are_order_independent() {
    let first = unexpected_ack("0/16B9000", "schema-v2", true, "accepted");
    let second = unexpected_ack("0/16BA000", "schema-v1", false, "rejected later ack");
    let forward = unexpected_duplicate_summary(vec![first.clone(), second.clone()]);
    let reverse = unexpected_duplicate_summary(vec![second, first]);

    assert_eq!(forward.release_dml, reverse.release_dml);
    assert_eq!(forward.unexpected_sinks, reverse.unexpected_sinks);
    assert_eq!(forward.release_blockers, reverse.release_blockers);
    assert_eq!(forward.release_blocker_codes, reverse.release_blocker_codes);
    assert_eq!(forward.release_actions, reverse.release_actions);
    assert_eq!(forward.sink_evidence, reverse.sink_evidence);

    let unexpected = forward
        .sink_evidence
        .iter()
        .find(|evidence| evidence.sink == "typo_target_postgres")
        .expect("unexpected evidence");
    assert_eq!(unexpected.ack_lsn.as_deref(), Some("0/16BA000"));
    assert_eq!(unexpected.schema_version.as_deref(), Some("schema-v1"));
    assert_eq!(unexpected.accepted, Some(false));
    assert_eq!(unexpected.detail.as_deref(), Some("rejected later ack"));
}

fn unexpected_duplicate_summary(acks: Vec<DdlBarrierAck>) -> DdlBarrierSummary {
    DdlBarrierSummary::from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-unexpected-duplicate".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        },
        acks,
    )
}

fn unexpected_ack(
    ack_lsn: &str,
    schema_version: &str,
    accepted: bool,
    detail: &str,
) -> DdlBarrierAck {
    DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-unexpected-duplicate".to_string(),
        sink: "typo_target_postgres".to_string(),
        ack_lsn: ack_lsn.to_string(),
        schema_version: schema_version.to_string(),
        accepted,
        detail: detail.to_string(),
    }
}
