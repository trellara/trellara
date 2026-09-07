use super::*;

#[tokio::test]
async fn ddl_barrier_summary_blocks_until_all_required_sinks_ack() {
    let store = InMemoryCheckpointStore::new();
    let flow = ddl_flow();
    let barrier = DdlBarrier {
        source_id: flow.source_id.clone(),
        database_id: "retail".to_string(),
        dataset_id: flow.dataset_id.clone(),
        barrier_id: "ddl-barrier-abc".to_string(),
        barrier_lsn: "0/16B8000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
        required_sinks: vec![
            "target_postgres".to_string(),
            "raw_cdc_lake".to_string(),
            "spark_derived_views".to_string(),
        ],
        requires_global_partition_pause: false,
    };

    store
        .record_ddl_barrier(barrier)
        .await
        .expect("record barrier");
    store
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: flow.source_id.clone(),
            database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-abc".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: target_postgres_detail(),
        })
        .await
        .expect("record target ack");

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-abc")
        .await
        .expect("load summary")
        .expect("summary");

    assert!(!summary.release_dml);
    assert_eq!(summary.required_sink_count, 3);
    assert_eq!(summary.acked_sinks, vec!["target_postgres"]);
    assert_eq!(
        summary.pending_sinks,
        vec!["raw_cdc_lake", "spark_derived_views"]
    );
    assert_eq!(
        summary.release_blockers,
        vec!["pending required sink acknowledgements: raw_cdc_lake, spark_derived_views"]
    );
    assert_eq!(summary.release_blocker_codes, vec!["pending_required_ack"]);
    assert_eq!(summary.release_blocker_details.len(), 1);
    assert_eq!(
        summary.release_blocker_details[0].code,
        "pending_required_ack"
    );
    assert_eq!(
        summary.release_blocker_details[0].message,
        "pending required sink acknowledgements"
    );
    assert_eq!(
        summary.release_blocker_details[0].sinks,
        vec!["raw_cdc_lake", "spark_derived_views"]
    );
    assert_eq!(
        summary.release_blocker_details[0].evidence,
        "barrier_id=ddl-barrier-abc barrier_lsn=0/16B8000 schema_version=schema-v2 pending_sinks=raw_cdc_lake, spark_derived_views"
    );
    assert_eq!(summary.release_actions.len(), 1);
    assert_eq!(summary.release_actions[0].code, "record_required_sink_ack");
    assert_eq!(
        summary.release_actions[0].sinks,
        vec!["raw_cdc_lake", "spark_derived_views"]
    );
    assert!(summary.release_actions[0]
        .command
        .contains("--ack-lsn 0/16B8000"));
    assert!(summary.release_actions[0]
        .reason
        .contains("post-DDL DML remains invisible"));
    assert!(summary.release_gates.iter().any(|gate| {
        gate.name == "schema_barrier_recorded"
            && gate.satisfied
            && gate.evidence.contains("barrier_lsn")
    }));
    assert!(summary.release_gates.iter().any(|gate| {
        gate.name == "required_sink_acknowledgements"
            && !gate.satisfied
            && gate.evidence.contains("raw_cdc_lake")
    }));
    assert!(summary.release_gates.iter().any(|gate| {
        gate.name == "post_ddl_dml_release"
            && !gate.satisfied
            && gate.evidence.contains("release_dml is false")
    }));
    assert!(summary.rejected_sinks.is_empty());
    assert!(summary.sink_evidence.iter().any(|evidence| {
        evidence.sink == "target_postgres"
            && evidence.status == "acked"
            && evidence.ack_lsn.as_deref() == Some("0/16B8000")
            && evidence.schema_version.as_deref() == Some("schema-v2")
            && evidence.accepted == Some(true)
            && evidence.detail.as_deref() == Some(target_postgres_detail().as_str())
            && evidence.release_eligible
    }));
    assert!(summary.sink_evidence.iter().any(|evidence| {
        evidence.sink == "raw_cdc_lake"
            && evidence.status == "pending"
            && evidence.ack_lsn.is_none()
            && !evidence.release_eligible
    }));
}

#[tokio::test]
async fn ddl_barrier_summary_scopes_same_barrier_id_by_database() {
    let store = InMemoryCheckpointStore::new();
    let retail = DdlBarrierLookup::new("source-a", "retail", "sales");
    let analytics = DdlBarrierLookup::new("source-a", "analytics", "sales");

    store
        .record_ddl_barrier(DdlBarrier {
            source_id: retail.source_id.clone(),
            database_id: retail.database_id.clone(),
            dataset_id: retail.dataset_id.clone(),
            barrier_id: "ddl-barrier-shared".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "retail-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect("record retail barrier");
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: analytics.source_id.clone(),
            database_id: analytics.database_id.clone(),
            dataset_id: analytics.dataset_id.clone(),
            barrier_id: "ddl-barrier-shared".to_string(),
            barrier_lsn: "0/16C8000".to_string(),
            schema_version: "analytics-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["raw_cdc_lake".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect("record analytics barrier");

    store
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: retail.source_id.clone(),
            database_id: retail.database_id.clone(),
            dataset_id: retail.dataset_id.clone(),
            barrier_id: "ddl-barrier-shared".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "retail-v2".to_string(),
            accepted: true,
            detail: target_postgres_detail(),
        })
        .await
        .expect("record retail ack");

    let retail_summary = store
        .ddl_barrier_summary(&retail, "ddl-barrier-shared")
        .await
        .expect("retail summary")
        .expect("retail barrier");
    let analytics_summary = store
        .ddl_barrier_summary(&analytics, "ddl-barrier-shared")
        .await
        .expect("analytics summary")
        .expect("analytics barrier");

    assert!(retail_summary.release_dml);
    assert_eq!(retail_summary.database_id, "retail");
    assert_eq!(analytics_summary.database_id, "analytics");
    assert!(!analytics_summary.release_dml);
    assert_eq!(analytics_summary.pending_sinks, vec!["raw_cdc_lake"]);
}

#[tokio::test]
async fn ddl_barrier_summary_parses_propagation_policy_evidence() {
    let store = InMemoryCheckpointStore::new();
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-policy".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: format!("source commit LSN is the DDL barrier; propagation_boundary=source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks; propagation_decisions=auto_apply:1,manual_review:1,unsupported:0,target_ack_required:2; propagation_policy_sha256={}; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn", "a".repeat(64)),
            required_sinks: vec!["target_postgres".to_string()],
            requires_global_partition_pause: false,
        })
        .await
        .expect("record DDL barrier");

    let summary = store
        .ddl_barrier_summary(
            &DdlBarrierLookup::new("source-a", "retail", "sales"),
            "ddl-barrier-policy",
        )
        .await
        .expect("load summary")
        .expect("summary");

    assert_eq!(
        summary.propagation_boundary.as_deref(),
        Some("source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks")
    );
    assert_eq!(
        summary.propagation_decisions,
        vec![
            "auto_apply:1".to_string(),
            "manual_review:1".to_string(),
            "unsupported:0".to_string(),
            "target_ack_required:2".to_string(),
        ]
    );
    assert_eq!(summary.propagation_policy_sha256, Some("a".repeat(64)));
}

#[test]
fn ddl_barrier_summary_blocks_release_without_required_sinks_contract() {
    let summary = DdlBarrierSummary::from_barrier_and_acks(
        DdlBarrier {
            source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-no-sinks".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: Vec::new(),
            requires_global_partition_pause: false,
        },
        Vec::new(),
    );

    assert!(!summary.release_dml);
    assert_eq!(summary.required_sink_count, 0);
    assert_eq!(
        summary.release_blockers,
        vec!["missing required sink acknowledgements contract"]
    );
    assert_eq!(
        summary.release_blocker_codes,
        vec!["missing_required_sinks"]
    );
    assert_eq!(summary.release_blocker_details.len(), 1);
    assert_eq!(
        summary.release_blocker_details[0].code,
        "missing_required_sinks"
    );
    assert!(summary.release_gates.iter().any(|gate| {
        gate.name == "required_sink_acknowledgements"
            && !gate.satisfied
            && gate.evidence.contains("missing required sink")
    }));
    assert!(summary.release_gates.iter().any(|gate| {
        gate.name == "post_ddl_dml_release"
            && !gate.satisfied
            && gate.evidence.contains("release_dml is false")
    }));
}

#[tokio::test]
async fn ddl_barrier_summary_structures_pending_partition_visibility_blocker() {
    let store = InMemoryCheckpointStore::new();
    let flow = ddl_flow();
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: flow.source_id.clone(),
        database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-partition-pause".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec![
                "target_postgres".to_string(),
                "partition_visibility".to_string(),
            ],
            requires_global_partition_pause: true,
        })
        .await
        .expect("record barrier");
    store
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: flow.source_id.clone(),
            database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-partition-pause".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: target_postgres_detail(),
        })
        .await
        .expect("record target ack");

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-partition-pause")
        .await
        .expect("load summary")
        .expect("summary");

    assert!(!summary.release_dml);
    assert_eq!(summary.pending_sinks, vec!["partition_visibility"]);
    assert_eq!(
        summary.release_blocker_codes,
        vec!["partition_visibility_not_released", "pending_required_ack"]
    );
    assert_eq!(
        summary
            .release_blocker_details
            .iter()
            .map(|blocker| blocker.code.as_str())
            .collect::<Vec<_>>(),
        vec!["pending_required_ack", "partition_visibility_not_released"]
    );
    assert_eq!(
        summary.release_blocker_details[1].sinks,
        vec!["partition_visibility"]
    );
    assert_eq!(
        summary.release_blocker_details[1].evidence,
        "barrier_id=ddl-barrier-partition-pause barrier_lsn=0/16B8000 schema_version=schema-v2 required_sink=partition_visibility"
    );
}

fn target_postgres_detail() -> String {
    target_postgres_ddl_ack_detail_with_digests(1, &"a".repeat(64), &["b".repeat(64)])
}
