use super::*;

#[tokio::test]
async fn target_ddl_barrier_releases_after_matching_target_ack() {
    let store = InMemoryCheckpointStore::new();
    let envelope = ddl_envelope();
    let barrier = target_ddl_barrier_from_envelope(&envelope)
        .expect("DDL barrier")
        .expect("DDL events");
    let flow = DdlBarrierLookup::from_barrier(&barrier);
    let barrier_id = barrier.barrier_id.clone();
    let schema_version = barrier.schema_version.clone();

    store
        .record_ddl_barrier(barrier)
        .await
        .expect("record DDL barrier");

    let pending = store
        .ddl_barrier_summary(&flow, &barrier_id)
        .await
        .expect("pending summary")
        .expect("summary");
    assert!(!pending.release_dml);
    assert_eq!(pending.pending_sinks, vec!["target_postgres"]);

    TargetDdlApplyOutcome {
        barrier_id: barrier_id.clone(),
        applied_statements: 1,
        release_gate: "post_ddl_dml_release".to_string(),
        plan_sha256: "a".repeat(64),
        statement_sha256s: vec!["b".repeat(64)],
    }
    .target_ack_evidence_for_database("source", "retail", "sales", "0/16B6C50", schema_version)
    .expect("target ack evidence")
    .record_barrier_ack(&store)
    .await
    .expect("record target ack");

    let released = store
        .ddl_barrier_summary(&flow, &barrier_id)
        .await
        .expect("released summary")
        .expect("summary");
    assert!(released.release_dml);
    assert!(released.release_blockers.is_empty());
    assert_eq!(released.acked_sinks, vec!["target_postgres"]);
}

#[tokio::test]
async fn partitioned_target_ddl_barrier_waits_for_partition_visibility_ack() {
    let store = InMemoryCheckpointStore::new();
    let envelope = ddl_envelope();
    let barrier = target_ddl_barrier_from_envelope_with_requirements(
        &envelope,
        TargetDdlBarrierRequirements::partitioned_scale(vec!["target_postgres".to_string()]),
    )
    .expect("DDL barrier")
    .expect("DDL events");
    let flow = DdlBarrierLookup::from_barrier(&barrier);
    let barrier_id = barrier.barrier_id.clone();
    let schema_version = barrier.schema_version.clone();

    store
        .record_ddl_barrier(barrier)
        .await
        .expect("record DDL barrier");
    TargetDdlApplyOutcome {
        barrier_id: barrier_id.clone(),
        applied_statements: 1,
        release_gate: "post_ddl_dml_release".to_string(),
        plan_sha256: "a".repeat(64),
        statement_sha256s: vec!["b".repeat(64)],
    }
    .target_ack_evidence_for_database("source", "retail", "sales", "0/16B6C50", schema_version)
    .expect("target ack evidence")
    .record_barrier_ack(&store)
    .await
    .expect("record target ack");

    let summary = store
        .ddl_barrier_summary(&flow, &barrier_id)
        .await
        .expect("summary")
        .expect("summary");
    assert!(!summary.release_dml);
    assert_eq!(summary.acked_sinks, vec!["target_postgres"]);
    assert_eq!(summary.pending_sinks, vec!["partition_visibility"]);
    assert!(summary.requires_global_partition_pause);
}

#[tokio::test]
async fn record_target_ddl_barrier_from_envelope_returns_pending_summary() {
    let store = InMemoryCheckpointStore::new();
    let envelope = ddl_envelope();

    let summary = record_target_ddl_barrier_from_envelope(
        &store,
        &envelope,
        TargetDdlBarrierRequirements::from_required_sinks(["raw_cdc_lake"], false),
    )
    .await
    .expect("record DDL barrier")
    .expect("DDL summary");

    assert_eq!(
        summary.barrier_id,
        "source:retail:sales:tx-ddl:0/16B6C50:ddl"
    );
    assert_eq!(
        summary.pending_sinks,
        vec!["raw_cdc_lake", "target_postgres"]
    );
    assert!(!summary.release_dml);
    assert!(summary
        .release_gates
        .iter()
        .any(|gate| gate.name == "post_ddl_dml_release" && !gate.satisfied));
}

#[test]
fn target_ddl_barrier_from_envelope_returns_none_without_ddl_events() {
    assert!(
        target_ddl_barrier_from_envelope(&envelope("tx-1", "0/16B6C50"))
            .expect("barrier")
            .is_none()
    );
}

#[tokio::test]
async fn record_target_ddl_barrier_from_envelope_returns_none_without_ddl_events() {
    let store = InMemoryCheckpointStore::new();

    let summary = record_target_ddl_barrier_from_envelope(
        &store,
        &envelope("tx-1", "0/16B6C50"),
        TargetDdlBarrierRequirements::target_postgres_only(),
    )
    .await
    .expect("record DDL barrier");

    assert!(summary.is_none());
}
