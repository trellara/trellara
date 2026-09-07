use super::*;

#[tokio::test]
async fn ddl_barrier_summary_releases_after_matching_sink_acks() {
    let store = InMemoryCheckpointStore::new();
    let flow = ddl_flow();
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: flow.source_id.clone(),
        database_id: "retail".to_string(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-release".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec![
                "target_postgres".to_string(),
                "raw_cdc_lake".to_string(),
                "partition_visibility".to_string(),
            ],
            requires_global_partition_pause: true,
        })
        .await
        .expect("record barrier");

    for sink in ["target_postgres", "raw_cdc_lake", "partition_visibility"] {
        store
            .record_ddl_barrier_ack(DdlBarrierAck {
                source_id: flow.source_id.clone(),
                database_id: "retail".to_string(),
                dataset_id: flow.dataset_id.clone(),
                barrier_id: "ddl-barrier-release".to_string(),
                sink: sink.to_string(),
                ack_lsn: "0/16B9000".to_string(),
                schema_version: "schema-v2".to_string(),
                accepted: true,
                detail: release_detail(sink),
            })
            .await
            .expect("record ack");
    }

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-release")
        .await
        .expect("load summary")
        .expect("summary");

    assert!(summary.release_dml);
    assert!(summary.release_blockers.is_empty());
    assert!(summary.release_blocker_codes.is_empty());
    assert!(summary.release_blocker_details.is_empty());
    assert!(summary.release_actions.is_empty());
    assert!(summary.pending_sinks.is_empty());
    assert!(summary.rejected_sinks.is_empty());
    assert!(summary.requires_global_partition_pause);
    assert!(summary.release_gates.iter().any(|gate| {
        gate.name == "partition_visibility_watermark"
            && gate.satisfied
            && gate.evidence.contains("partition_visibility ACK")
    }));
    assert!(summary.release_gates.iter().any(|gate| {
        gate.name == "post_ddl_dml_release"
            && gate.satisfied
            && gate.evidence.contains("no release_blockers")
    }));
    assert!(summary.release_gates.iter().any(|gate| {
        gate.name == "required_sink_acknowledgements"
            && gate.satisfied
            && gate.evidence.contains("accepted")
            && gate.evidence.contains("schema_version")
            && gate.evidence.contains("detail evidence")
    }));
    assert_eq!(summary.sink_evidence.len(), 3);
    assert!(summary
        .sink_evidence
        .iter()
        .all(|evidence| evidence.release_eligible && evidence.accepted == Some(true)));
}

fn release_detail(sink: &str) -> String {
    match sink {
        "target_postgres" => {
            target_postgres_ddl_ack_detail_with_digests(1, &"a".repeat(64), &["b".repeat(64)])
        }
        "partition_visibility" => {
            partition_visibility_ddl_ack_detail(3, 3, "0/16B9000", "0/16B9000", &"c".repeat(64))
        }
        "raw_cdc_lake" => format!(
            "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest={}; release_gate=post_ddl_dml_release",
            "d".repeat(64)
        ),
        _ => "accepted".to_string(),
    }
}
