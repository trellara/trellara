use super::*;

#[tokio::test]
async fn ddl_barrier_ack_rejects_sink_with_surrounding_whitespace() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-spaced-ack".to_string(),
            sink: " target_postgres ".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: "target applied schema barrier".to_string(),
        })
        .await
        .expect_err("spaced ack sink");

    assert!(error
        .to_string()
        .contains("ack sink must not contain surrounding whitespace"));
}

#[tokio::test]
async fn ddl_barrier_ack_rejects_schema_version_with_surrounding_whitespace() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-spaced-ack-schema".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: " schema-v2 ".to_string(),
            accepted: true,
            detail: "target applied schema barrier".to_string(),
        })
        .await
        .expect_err("spaced ack schema version");

    assert!(error
        .to_string()
        .contains("schema_version must not contain surrounding whitespace"));
}

#[tokio::test]
async fn ddl_barrier_ack_requires_detail_evidence() {
    for detail in ["", "   "] {
        let error = InMemoryCheckpointStore::new()
            .record_ddl_barrier_ack(DdlBarrierAck {
                source_id: "source-a".to_string(),
                database_id: "retail".to_string(),
                dataset_id: "sales".to_string(),
                barrier_id: "ddl-barrier-missing-detail".to_string(),
                sink: "target_postgres".to_string(),
                ack_lsn: "0/16B9000".to_string(),
                schema_version: "schema-v2".to_string(),
                accepted: true,
                detail: detail.to_string(),
            })
            .await
            .expect_err("missing detail");

        assert!(error.to_string().contains("detail evidence"));
    }
}

#[tokio::test]
async fn ddl_barrier_ack_rejects_detail_evidence_with_surrounding_whitespace() {
    let error = InMemoryCheckpointStore::new()
        .record_ddl_barrier_ack(DdlBarrierAck {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            barrier_id: "ddl-barrier-spaced-detail".to_string(),
            sink: "target_postgres".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            accepted: true,
            detail: " target applied schema barrier ".to_string(),
        })
        .await
        .expect_err("spaced ack detail");

    assert!(error
        .to_string()
        .contains("detail evidence must not contain surrounding whitespace"));
}

#[tokio::test]
async fn ddl_barrier_ack_rejects_identity_fields_with_surrounding_whitespace() {
    for (source_id, dataset_id, barrier_id, field) in [
        (
            " source-a ",
            "sales",
            "ddl-barrier-spaced-source",
            "source_id",
        ),
        (
            "source-a",
            " sales ",
            "ddl-barrier-spaced-dataset",
            "dataset_id",
        ),
        ("source-a", "sales", " ddl-barrier-spaced-id ", "barrier_id"),
    ] {
        let error = InMemoryCheckpointStore::new()
            .record_ddl_barrier_ack(DdlBarrierAck {
                source_id: source_id.to_string(),
                database_id: "retail".to_string(),
                dataset_id: dataset_id.to_string(),
                barrier_id: barrier_id.to_string(),
                sink: "target_postgres".to_string(),
                ack_lsn: "0/16B9000".to_string(),
                schema_version: "schema-v2".to_string(),
                accepted: true,
                detail: "target applied schema barrier".to_string(),
            })
            .await
            .expect_err("spaced ack identity");

        assert!(error.to_string().contains(field));
        assert!(error
            .to_string()
            .contains("must not contain surrounding whitespace"));
    }
}

#[tokio::test]
async fn ddl_barrier_ack_rejects_sink_scoped_lsn_rewind() {
    let store = InMemoryCheckpointStore::new();
    let first = DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-ack-rewind".to_string(),
        sink: "target_postgres".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: "target applied schema barrier".to_string(),
    };
    store
        .record_ddl_barrier_ack(first.clone())
        .await
        .expect("record first ack");

    let mut stale = first;
    stale.ack_lsn = "0/16B8000".to_string();
    stale.detail = "stale target ack".to_string();

    let error = store
        .record_ddl_barrier_ack(stale)
        .await
        .expect_err("stale ack rewind");

    assert!(error.to_string().contains("cannot move backward"));
}

#[tokio::test]
async fn ddl_barrier_ack_allows_same_lsn_idempotent_update() {
    let store = InMemoryCheckpointStore::new();
    let ack = DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-ack-idempotent".to_string(),
        sink: "target_postgres".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: "target applied schema barrier".to_string(),
    };
    store
        .record_ddl_barrier_ack(ack.clone())
        .await
        .expect("record first ack");
    store
        .record_ddl_barrier_ack(DdlBarrierAck {
            detail: "target applied schema barrier again".to_string(),
            ..ack
        })
        .await
        .expect("same LSN update");
}

#[tokio::test]
async fn ddl_barrier_ack_rejects_same_lsn_schema_version_conflict() {
    let store = InMemoryCheckpointStore::new();
    let ack = DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-ack-schema-conflict".to_string(),
        sink: "target_postgres".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: "target applied schema barrier".to_string(),
    };
    store
        .record_ddl_barrier_ack(ack.clone())
        .await
        .expect("record first ack");

    let error = store
        .record_ddl_barrier_ack(DdlBarrierAck {
            schema_version: "schema-v1".to_string(),
            detail: "same LSN but stale schema".to_string(),
            ..ack
        })
        .await
        .expect_err("same-LSN schema conflict");

    assert!(error.to_string().contains("conflicts with existing ACK"));
}

#[tokio::test]
async fn ddl_barrier_ack_rejects_same_lsn_acceptance_conflict() {
    let store = InMemoryCheckpointStore::new();
    let ack = DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        barrier_id: "ddl-barrier-ack-decision-conflict".to_string(),
        sink: "target_postgres".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: "target applied schema barrier".to_string(),
    };
    store
        .record_ddl_barrier_ack(ack.clone())
        .await
        .expect("record first ack");

    let error = store
        .record_ddl_barrier_ack(DdlBarrierAck {
            accepted: false,
            detail: "same LSN but sink rejected".to_string(),
            ..ack
        })
        .await
        .expect_err("same-LSN decision conflict");

    assert!(error.to_string().contains("conflicts with existing ACK"));
}
