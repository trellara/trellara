use super::*;

#[test]
fn ddl_barrier_ack_rejects_missing_typed_evidence() {
    let error =
        ddl_barrier_ack_from_args(&config(), &base_args("spark_derived_views")).expect_err("error");

    assert!(error.to_string().contains("requires --template-digest"));
}

#[test]
fn ddl_barrier_ack_rejects_target_postgres_without_plan_digest() {
    let error =
        ddl_barrier_ack_from_args(&config(), &base_args("target_postgres")).expect_err("error");

    assert!(error.to_string().contains("requires --plan-sha256"));
}

#[test]
fn ddl_barrier_ack_rejects_malformed_barrier_id() {
    for barrier_id in ["", "   ", " ddl-barrier-123", "ddl-barrier-123 "] {
        let mut args = base_args("target_postgres");
        args.barrier_id = barrier_id.to_string();

        let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("bad barrier id");

        assert!(
            error.to_string().contains("--barrier-id"),
            "unexpected error for {barrier_id:?}: {error}"
        );
    }
}

#[test]
fn ddl_barrier_ack_rejects_empty_typed_evidence() {
    let mut args = base_args("raw_cdc_lake");
    args.epoch_id = Some("   ".to_string());
    args.metadata_table = Some("_trellara_raw_cdc_epochs".to_string());
    args.partition_metadata_table = Some("_trellara_epoch_partitions".to_string());
    args.manifest_digest =
        Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string());

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("empty typed evidence");

    assert!(error.to_string().contains("--epoch-id must not be empty"));
}

#[test]
fn ddl_barrier_ack_rejects_missing_lake_manifest_digest() {
    let mut args = base_args("raw_cdc_lake");
    args.epoch_id = Some("epoch-2026-08-16T06".to_string());
    args.metadata_table = Some("_trellara_raw_cdc_epochs".to_string());
    args.partition_metadata_table = Some("_trellara_epoch_partitions".to_string());

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("missing manifest digest");

    assert!(error.to_string().contains("requires --manifest-digest"));
}

#[test]
fn ddl_barrier_ack_rejects_malformed_lake_manifest_digest() {
    let mut args = base_args("raw_cdc_lake");
    args.epoch_id = Some("epoch-2026-08-16T06".to_string());
    args.metadata_table = Some("_trellara_raw_cdc_epochs".to_string());
    args.partition_metadata_table = Some("_trellara_epoch_partitions".to_string());
    args.manifest_digest = Some("not-a-digest".to_string());

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("malformed digest");

    assert!(error
        .to_string()
        .contains("must be a 64-character SHA-256 hex digest"));
}

#[test]
fn ddl_barrier_ack_rejects_missing_lake_partition_metadata_table() {
    let mut args = base_args("raw_cdc_lake");
    args.epoch_id = Some("epoch-2026-08-16T06".to_string());
    args.metadata_table = Some("_trellara_raw_cdc_epochs".to_string());
    args.manifest_digest =
        Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string());

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("missing partitions table");

    assert!(error
        .to_string()
        .contains("requires --partition-metadata-table"));
}

#[test]
fn ddl_barrier_ack_rejects_spaced_typed_evidence() {
    let mut args = base_args("spark_derived_views");
    args.template_digest =
        Some(" 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string());
    args.accepted_by = Some("platform-review".to_string());
    args.view_count = Some(2);

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("spaced typed evidence");

    assert!(error
        .to_string()
        .contains("--template-digest must not contain surrounding whitespace"));
}

#[test]
fn ddl_barrier_ack_rejects_malformed_spark_template_digest() {
    let mut args = base_args("spark_derived_views");
    args.template_digest = Some("sha256:templates-v2".to_string());
    args.accepted_by = Some("platform-review".to_string());
    args.view_count = Some(2);

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("malformed digest");

    assert!(error
        .to_string()
        .contains("must be a 64-character SHA-256 hex digest"));
}

#[test]
fn ddl_barrier_ack_rejects_spaced_partition_barrier_lsn() {
    let mut args = base_args("partition_visibility");
    args.barrier_lsn = Some(" 0/16B9000".to_string());
    args.expected_partition_count = Some(2);
    args.partition_durable_lsns = vec!["0=0/16B9000".to_string(), "1=0/16BA000".to_string()];
    args.partition_applied_lsns = vec!["0=0/16B9000".to_string(), "1=0/16BA000".to_string()];

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("spaced barrier lsn");

    assert!(error
        .to_string()
        .contains("--barrier-lsn must not contain surrounding whitespace"));
}

#[test]
fn ddl_barrier_ack_rejects_spaced_partition_watermark() {
    let mut args = base_args("partition_visibility");
    args.barrier_lsn = Some("0/16B9000".to_string());
    args.expected_partition_count = Some(2);
    args.partition_durable_lsns = vec!["0=0/16B9000".to_string(), "1=0/16BA000".to_string()];
    args.partition_applied_lsns = vec!["0 =0/16B9000".to_string(), "1=0/16BA000".to_string()];

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("spaced partition");

    assert!(error
        .to_string()
        .contains("partition-applied-lsn 0 =0/16B9000 must not contain surrounding whitespace"));
}

#[test]
fn ddl_barrier_ack_rejects_partition_watermark_without_durable_evidence() {
    let mut args = base_args("partition_visibility");
    args.barrier_lsn = Some("0/16B9000".to_string());
    args.expected_partition_count = Some(2);
    args.partition_applied_lsns = vec!["0=0/16B9000".to_string(), "1=0/16BA000".to_string()];

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("missing durable lsn");

    assert!(error
        .to_string()
        .contains("requires --partition-durable-lsn"));
}

#[test]
fn ddl_barrier_ack_rejects_partition_watermark_mismatched_durable_applied_ids() {
    let mut args = base_args("partition_visibility");
    args.barrier_lsn = Some("0/16B9000".to_string());
    args.expected_partition_count = Some(2);
    args.partition_durable_lsns = vec!["0=0/16B9000".to_string(), "2=0/16BA000".to_string()];
    args.partition_applied_lsns = vec!["0=0/16B9000".to_string(), "1=0/16BA000".to_string()];

    let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("mismatched partition ids");

    assert!(error.to_string().contains("cover the same partition ids"));
}

#[test]
fn ddl_barrier_ack_rejects_manual_ack_without_detail_evidence() {
    for accepted in [true, false] {
        let mut args = base_args("custom_sink");
        args.accepted = accepted;
        args.detail = "   ".to_string();

        let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("missing detail");

        assert!(error.to_string().contains("--detail must include evidence"));
    }
}

#[test]
fn ddl_barrier_ack_rejects_sink_with_surrounding_whitespace() {
    for sink in [" target_postgres", "target_postgres ", " raw_cdc_lake "] {
        let args = base_args(sink);

        let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("spaced sink");

        assert!(error
            .to_string()
            .contains("--sink must not contain surrounding whitespace"));
    }
}

#[test]
fn ddl_barrier_ack_rejects_empty_sink() {
    for sink in ["", "   "] {
        let args = base_args(sink);

        let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("empty sink");

        assert!(error.to_string().contains("--sink must not be empty"));
    }
}

#[test]
fn ddl_barrier_ack_rejects_malformed_ack_lsn_before_sink_dispatch() {
    for ack_lsn in ["", "   ", " 0/16B9000", "0/16B9000 ", "not-a-lsn", "0/0"] {
        let mut args = base_args("target_postgres");
        args.ack_lsn = ack_lsn.to_string();
        args.accepted = false;
        args.detail = "target rejected schema".to_string();

        let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("bad ack lsn");

        assert!(
            error.to_string().contains("--ack-lsn"),
            "unexpected error for {ack_lsn:?}: {error}"
        );
    }
}

#[test]
fn ddl_barrier_ack_rejects_malformed_schema_version_before_sink_dispatch() {
    for schema_version in ["", "   ", " schema-v2", "schema-v2 "] {
        let mut args = base_args("target_postgres");
        args.schema_version = schema_version.to_string();
        args.accepted = false;
        args.detail = "target rejected schema".to_string();

        let error = ddl_barrier_ack_from_args(&config(), &args).expect_err("bad schema version");

        assert!(
            error.to_string().contains("--schema-version"),
            "unexpected error for {schema_version:?}: {error}"
        );
    }
}
