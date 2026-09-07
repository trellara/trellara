use super::*;

#[tokio::test]
async fn partitioned_barrier_codes_partition_visibility_hold_before_ack() {
    let store = InMemoryCheckpointStore::default();
    let flow = DdlBarrierLookup::new("source-a", "retail", "dataset-a");
    store
        .record_ddl_barrier(partitioned_barrier())
        .await
        .unwrap();
    for ack in [target_ack(), raw_lake_ack(), spark_ack()] {
        store.record_ddl_barrier_ack(ack).await.unwrap();
    }

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-partitioned")
        .await
        .unwrap()
        .expect("summary");

    assert!(!summary.release_dml);
    assert_eq!(summary.pending_sinks, vec!["partition_visibility"]);
    assert_eq!(
        summary.release_blocker_codes,
        vec!["partition_visibility_not_released", "pending_required_ack"]
    );
    assert_eq!(summary.release_actions.len(), 2);
    assert!(summary
        .release_actions
        .iter()
        .any(|action| action.code == "record_partition_visibility_ack"
            && action.command.contains("trellara partition-watermarks")));
}

#[tokio::test]
async fn partition_visibility_ddl_ack_completes_partitioned_barrier() {
    let store = InMemoryCheckpointStore::default();
    let flow = DdlBarrierLookup::new("source-a", "retail", "dataset-a");
    store
        .record_ddl_barrier(partitioned_barrier())
        .await
        .unwrap();
    for ack in [target_ack(), raw_lake_ack(), spark_ack()] {
        store.record_ddl_barrier_ack(ack).await.unwrap();
    }
    partition_visibility_ddl_ack_evidence(request(complete_watermarks()))
        .unwrap()
        .record_barrier_ack(&store)
        .await
        .unwrap();

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-partitioned")
        .await
        .unwrap()
        .expect("summary");

    assert!(summary.release_dml);
    assert!(summary
        .release_gates
        .iter()
        .any(|gate| { gate.name == "partition_visibility_watermark" && gate.satisfied }));
    assert!(summary.release_actions.is_empty());
}

#[tokio::test]
async fn partition_visibility_ack_without_watermark_detail_does_not_release_barrier() {
    let store = InMemoryCheckpointStore::default();
    let flow = DdlBarrierLookup::new("source-a", "retail", "dataset-a");
    store
        .record_ddl_barrier(partitioned_barrier())
        .await
        .unwrap();
    for ack in [target_ack(), raw_lake_ack(), spark_ack()] {
        store.record_ddl_barrier_ack(ack).await.unwrap();
    }
    store
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "dataset-a".to_string(),
            barrier_id: "ddl-barrier-partitioned".to_string(),
            sink: "partition_visibility".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: "partition visibility accepted schema-v2".to_string(),
        })
        .await
        .unwrap();

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-partitioned")
        .await
        .unwrap()
        .expect("summary");

    assert!(!summary.release_dml);
    assert_eq!(summary.rejected_sinks, vec!["partition_visibility"]);
    assert_eq!(
        summary.release_blocker_codes,
        vec![
            "insufficient_partition_visibility_evidence",
            "partition_visibility_not_released"
        ]
    );
}
