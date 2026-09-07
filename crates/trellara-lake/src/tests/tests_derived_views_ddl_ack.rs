use super::*;
use trellara_checkpoint::{
    DdlBarrier, DdlBarrierAck, DdlBarrierLookup, DdlBarrierStore, InMemoryCheckpointStore,
};

const TEMPLATE_DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn valid_ack() -> SparkDerivedViewsDdlAckEvidence {
    spark_derived_views_ddl_ack_evidence(valid_request()).expect("spark derived DDL ack")
}

fn valid_request() -> SparkDerivedViewsDdlAckRequest {
    SparkDerivedViewsDdlAckRequest {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        template_digest: TEMPLATE_DIGEST.to_string(),
        accepted_by: "platform-review".to_string(),
        view_count: 2,
    }
}

#[test]
fn spark_derived_views_ddl_ack_converts_to_barrier_ack() {
    let ack = valid_ack().into_barrier_ack();

    assert_eq!(ack.sink, "spark_derived_views");
    assert_eq!(ack.ack_lsn, "0/16B9000");
    assert_eq!(ack.schema_version, "schema-v2");
    assert!(ack.accepted);
    assert!(ack.detail.contains(TEMPLATE_DIGEST));
    assert!(ack.detail.contains("platform-review"));
    assert!(ack.detail.contains("post_ddl_dml_release"));
}

#[test]
fn spark_derived_views_ddl_ack_rejects_invalid_evidence() {
    for ack_lsn in [
        "not-a-lsn",
        "0/0",
        "xyz/16B9000",
        "100000000/0",
        "0/100000000",
    ] {
        let mut request = valid_request();
        request.ack_lsn = ack_lsn.to_string();
        assert!(matches!(
            spark_derived_views_ddl_ack_evidence(request),
            Err(LakeError::InvalidDdlAckLsn { .. })
        ));
    }

    let mut missing_digest = valid_request();
    missing_digest.template_digest.clear();
    assert!(matches!(
        spark_derived_views_ddl_ack_evidence(missing_digest),
        Err(LakeError::MissingDdlAckField {
            field: "template_digest"
        })
    ));

    let mut invalid_digest = valid_request();
    invalid_digest.template_digest = "sha256:templates-v2".to_string();
    assert!(matches!(
        spark_derived_views_ddl_ack_evidence(invalid_digest),
        Err(LakeError::InvalidDdlAckField {
            field: "template_digest",
            ..
        })
    ));

    let mut zero_views = valid_request();
    zero_views.view_count = 0;
    assert!(matches!(
        spark_derived_views_ddl_ack_evidence(zero_views),
        Err(LakeError::InvalidDdlAckField {
            field: "view_count",
            ..
        })
    ));

    let mut spaced_schema = valid_request();
    spaced_schema.schema_version = " schema-v2 ".to_string();
    let error =
        spark_derived_views_ddl_ack_evidence(spaced_schema).expect_err("spaced schema version");
    assert!(matches!(
        error,
        LakeError::InvalidDdlAckField {
            field: "schema_version",
            ..
        }
    ));
    assert!(error
        .to_string()
        .contains("must not contain surrounding whitespace"));
}

#[test]
fn spark_derived_views_ddl_ack_rejects_fields_with_surrounding_whitespace() {
    for field in [
        "source_id",
        "dataset_id",
        "barrier_id",
        "ack_lsn",
        "schema_version",
        "template_digest",
        "accepted_by",
    ] {
        let mut request = valid_request();
        match field {
            "source_id" => request.source_id = " source-a".to_string(),
            "dataset_id" => request.dataset_id = "dataset-a ".to_string(),
            "barrier_id" => request.barrier_id = " ddl-barrier-123".to_string(),
            "ack_lsn" => request.ack_lsn = "0/16B9000 ".to_string(),
            "schema_version" => request.schema_version = " schema-v2".to_string(),
            "template_digest" => request.template_digest = format!(" {TEMPLATE_DIGEST}"),
            "accepted_by" => request.accepted_by = "platform-review ".to_string(),
            _ => unreachable!("test fields are exhaustive"),
        }

        assert!(matches!(
            spark_derived_views_ddl_ack_evidence(request),
            Err(LakeError::InvalidDdlAckField { field: actual, reason })
                if actual == field && reason == "must not contain surrounding whitespace"
        ));
    }
}

#[tokio::test]
async fn spark_derived_views_ack_completes_ddl_release_gate() {
    let store = InMemoryCheckpointStore::default();
    let flow = DdlBarrierLookup::new("source-a", "retail", "dataset-a");
    store.record_ddl_barrier(shared_barrier()).await.unwrap();
    for ack in [target_ack(), raw_lake_ack()] {
        store.record_ddl_barrier_ack(ack).await.unwrap();
    }
    valid_ack().record_barrier_ack(&store).await.unwrap();

    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-123")
        .await
        .unwrap()
        .expect("barrier summary");

    assert!(summary.release_dml);
    assert!(summary.pending_sinks.is_empty());
    assert!(summary.sink_evidence.iter().any(|evidence| {
        evidence.sink == "spark_derived_views"
            && evidence.accepted == Some(true)
            && evidence
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains(TEMPLATE_DIGEST))
    }));
}

fn shared_barrier() -> DdlBarrier {
    DdlBarrier {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        barrier_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
        required_sinks: vec![
            "target_postgres".to_string(),
            "raw_cdc_lake".to_string(),
            "spark_derived_views".to_string(),
        ],
        requires_global_partition_pause: false,
    }
}

fn target_ack() -> DdlBarrierAck {
    DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        sink: "target_postgres".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: trellara_checkpoint::target_postgres_ddl_ack_detail_with_digests(
            1,
            &"a".repeat(64),
            &["b".repeat(64)],
        ),
    }
}

fn raw_lake_ack() -> DdlBarrierAck {
    DdlBarrierAck {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        sink: "raw_cdc_lake".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: format!(
            "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest={}; release_gate=post_ddl_dml_release",
            "c".repeat(64)
        ),
    }
}
