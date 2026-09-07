use super::*;

#[test]
fn ddl_barrier_ack_builds_raw_cdc_lake_evidence() {
    let mut args = base_args("raw_cdc_lake");
    args.epoch_id = Some("epoch-2026-08-16T06".to_string());
    args.metadata_table = Some("_trellara_raw_cdc_epochs".to_string());
    args.partition_metadata_table = Some("_trellara_epoch_partitions".to_string());
    args.manifest_digest =
        Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string());

    let ack = ddl_barrier_ack_from_args(&config(), &args).expect("ack");

    assert_eq!(ack.sink, "raw_cdc_lake");
    assert_eq!(ack.source_id, "local-source");
    assert_eq!(ack.dataset_id, "retail-sales");
    assert!(ack.detail.contains("epoch-2026-08-16T06"));
    assert!(ack.detail.contains("_trellara_epoch_partitions"));
    assert!(ack
        .detail
        .contains("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"));
    assert!(ack.detail.contains("post_ddl_dml_release"));
}

#[test]
fn ddl_barrier_ack_builds_spark_derived_views_evidence() {
    let mut args = base_args("spark_derived_views");
    args.template_digest =
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string());
    args.accepted_by = Some("platform-review".to_string());
    args.view_count = Some(2);

    let ack = ddl_barrier_ack_from_args(&config(), &args).expect("ack");

    assert_eq!(ack.sink, "spark_derived_views");
    assert!(ack
        .detail
        .contains("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
    assert!(ack.detail.contains("platform-review"));
}

#[test]
fn ddl_barrier_ack_builds_partition_visibility_evidence() {
    let mut args = base_args("partition_visibility");
    args.barrier_lsn = Some("0/16B9000".to_string());
    args.expected_partition_count = Some(2);
    args.partition_durable_lsns = vec!["0=0/16B9000".to_string(), "1=0/16BA000".to_string()];
    args.partition_applied_lsns = vec!["0=0/16B9000".to_string(), "1=0/16BA000".to_string()];

    let ack = ddl_barrier_ack_from_args(&config(), &args).expect("ack");

    assert_eq!(ack.sink, "partition_visibility");
    assert_eq!(ack.ack_lsn, "0/16B9000");
    assert!(ack.detail.contains("2/2 partitions"));
    assert!(ack.detail.contains("global_durable_lsn 0/16B9000"));
    assert!(ack.detail.contains("partition_watermark_sha256="));
}

#[test]
fn ddl_barrier_ack_builds_target_postgres_digest_evidence() {
    let mut args = base_args("target_postgres");
    args.plan_sha256 = Some("a".repeat(64));
    args.statement_sha256s = vec!["b".repeat(64), "c".repeat(64)];

    let ack = ddl_barrier_ack_from_args(&config(), &args).expect("ack");

    assert_eq!(ack.sink, "target_postgres");
    assert!(ack.accepted);
    assert!(ack
        .detail
        .contains("target Postgres applied 2 DDL statements"));
    assert!(ack.detail.contains("plan_sha256="));
    assert!(ack.detail.contains("statement_sha256="));
    assert!(ack.detail.contains("release_gate=post_ddl_dml_release"));
}

#[test]
fn ddl_barrier_ack_builds_target_postgres_barrier_bound_evidence() {
    let mut args = base_args("target_postgres");
    args.plan_sha256 = Some("a".repeat(64));
    args.statement_sha256s = vec!["b".repeat(64)];
    args.barrier_lsn = Some("0/16B8000".to_string());

    let ack = ddl_barrier_ack_from_args(&config(), &args).expect("ack");

    assert_eq!(ack.sink, "target_postgres");
    assert!(ack.detail.contains("barrier_lsn=0/16B8000"));
}

#[test]
fn ddl_barrier_ack_preserves_rejected_typed_sink_ack() {
    let mut args = base_args("raw_cdc_lake");
    args.accepted = false;
    args.detail = "lake metadata did not include schema-v2".to_string();

    let ack = ddl_barrier_ack_from_args(&config(), &args).expect("ack");

    assert_eq!(ack.sink, "raw_cdc_lake");
    assert!(!ack.accepted);
    assert_eq!(ack.detail, "lake metadata did not include schema-v2");
    assert!(!ack.detail.contains("post_ddl_dml_release"));
}

#[test]
fn ddl_barrier_ack_keeps_manual_sink_path() {
    let mut args = base_args("target_postgres");
    args.accepted = false;
    args.detail = "target rejected schema".to_string();

    let ack = ddl_barrier_ack_from_args(&config(), &args).expect("ack");

    assert_eq!(ack.sink, "target_postgres");
    assert!(!ack.accepted);
    assert_eq!(ack.detail, "target rejected schema");
}
