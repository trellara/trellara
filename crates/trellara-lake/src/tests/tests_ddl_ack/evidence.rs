use super::*;

#[test]
fn raw_cdc_lake_ddl_ack_evidence_converts_to_barrier_ack() {
    let evidence = valid_ack();
    let ack = evidence.into_barrier_ack();

    assert_eq!(ack.source_id, "source-a");
    assert_eq!(ack.dataset_id, "dataset-a");
    assert_eq!(ack.barrier_id, "ddl-barrier-123");
    assert_eq!(ack.sink, "raw_cdc_lake");
    assert_eq!(ack.ack_lsn, "0/16B9000");
    assert_eq!(ack.schema_version, "schema-v2");
    assert!(ack.accepted);
    assert!(ack.detail.contains("epoch-2026-08-16T06"));
    assert!(ack.detail.contains("_trellara_epoch_partitions"));
    assert!(ack.detail.contains(MANIFEST_DIGEST));
    assert!(ack.detail.contains("schema-v2"));
    assert!(ack.detail.contains("post_ddl_dml_release"));
}
