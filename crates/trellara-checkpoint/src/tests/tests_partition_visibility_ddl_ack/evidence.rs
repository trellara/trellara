use super::*;

type WatermarkMutation = (&'static str, fn(&mut PartitionWatermarkSummary));
type RequestMutation = (&'static str, fn(&mut PartitionVisibilityDdlAckRequest));

#[test]
fn partition_visibility_ddl_ack_converts_complete_watermarks_to_barrier_ack() {
    let evidence =
        partition_visibility_ddl_ack_evidence(request(complete_watermarks())).expect("evidence");
    assert_eq!(evidence.durable_lsn, "0/16B9000");
    assert_eq!(evidence.partition_watermark_sha256.len(), 64);
    let ack = evidence.into_barrier_ack();

    assert_eq!(ack.sink, "partition_visibility");
    assert_eq!(ack.ack_lsn, "0/16B9000");
    assert_eq!(ack.schema_version, "schema-v2");
    assert!(ack.accepted);
    assert!(ack.detail.contains("2/2 partitions"));
    assert!(ack.detail.contains("global_durable_lsn 0/16B9000"));
    assert!(ack.detail.contains("global_applied_lsn 0/16B9000"));
    assert!(ack.detail.contains("partition_watermark_sha256="));
    assert!(ack.detail.contains("post_ddl_dml_release"));
}

#[test]
fn partition_visibility_ddl_ack_digest_changes_when_partition_evidence_changes() {
    let first = partition_watermark_summary_sha256(&complete_watermarks());
    let changed = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "dataset-a",
        2,
        vec![partition(0, "0/16BA000"), partition(1, "0/16BA000")],
    )
    .expect("changed partition watermarks");

    assert_ne!(first, partition_watermark_summary_sha256(&changed));
}

#[test]
fn partition_visibility_ddl_ack_digest_is_stable_for_partition_order() {
    let first = complete_watermarks();
    let mut reordered = first.clone();
    reordered.partitions.reverse();

    assert_eq!(
        partition_watermark_summary_sha256(&first),
        partition_watermark_summary_sha256(&reordered)
    );
}

#[test]
fn partition_visibility_ddl_ack_detail_requires_watermark_digest() {
    let legacy_detail = "partition visibility reached barrier with 2/2 partitions at global_durable_lsn 0/16B9000 and global_applied_lsn 0/16B9000; release_gate=post_ddl_dml_release";

    assert!(!partition_visibility_ddl_ack_detail_is_valid(
        legacy_detail,
        "0/16B9000",
        parse_lsn("0/16B9000")
    ));
}

#[test]
fn partition_visibility_ddl_ack_rejects_incomplete_or_lagging_watermarks() {
    let incomplete = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "dataset-a",
        2,
        vec![partition(0, "0/16BA000")],
    )
    .expect("incomplete watermarks");
    assert!(partition_visibility_ddl_ack_evidence(request(incomplete)).is_err());

    let lagging = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "dataset-a",
        2,
        vec![partition(0, "0/16B8FFF"), partition(1, "0/16BA000")],
    )
    .expect("lagging watermarks");
    assert!(partition_visibility_ddl_ack_evidence(request(lagging)).is_err());
}

#[test]
fn partition_visibility_ddl_ack_rejects_missing_global_applied_lsn_without_panic() {
    let mut watermarks = complete_watermarks();
    watermarks.global_applied_lsn = None;

    let error = partition_visibility_ddl_ack_evidence(request(watermarks))
        .expect_err("missing global applied watermark");

    assert!(error
        .to_string()
        .contains("partition visibility DDL ack is missing global_applied_lsn"));
}

#[test]
fn partition_visibility_ddl_ack_rejects_lagging_global_durable_lsn() {
    let mut watermarks = complete_watermarks();
    watermarks.global_durable_lsn = Some("0/16B8FFF".to_string());

    let error = partition_visibility_ddl_ack_evidence(request(watermarks))
        .expect_err("lagging durable watermark");

    assert!(error.to_string().contains("global_durable_lsn"));
    assert!(error.to_string().contains("must match"));
}

#[test]
fn partition_visibility_ddl_ack_rejects_inconsistent_watermark_summary() {
    let cases: [WatermarkMutation; 7] = [
        (
            "observed_partition_count",
            |watermarks: &mut PartitionWatermarkSummary| {
                watermarks.observed_partition_count = 1;
            },
        ),
        (
            "expected_partition_count",
            |watermarks: &mut PartitionWatermarkSummary| {
                watermarks.expected_partition_count = 3;
            },
        ),
        (
            "missing partitions",
            |watermarks: &mut PartitionWatermarkSummary| {
                watermarks.missing_partitions = vec![2];
            },
        ),
        (
            "global_durable_lsn",
            |watermarks: &mut PartitionWatermarkSummary| {
                watermarks.global_durable_lsn = None;
            },
        ),
        (
            "duplicate partition",
            |watermarks: &mut PartitionWatermarkSummary| {
                watermarks.partitions[1].partition_id = 0;
            },
        ),
        (
            "outside expected range",
            |watermarks: &mut PartitionWatermarkSummary| {
                watermarks.partitions[1].partition_id = 2;
            },
        ),
        (
            "global_applied_lsn must match",
            |watermarks: &mut PartitionWatermarkSummary| {
                watermarks.global_applied_lsn = Some("0/16BA000".to_string());
            },
        ),
    ];

    for (expected_message, mutate) in cases {
        let mut watermarks = complete_watermarks();
        mutate(&mut watermarks);

        let error = partition_visibility_ddl_ack_evidence(request(watermarks))
            .expect_err("inconsistent watermark summary");

        assert!(error.to_string().contains(expected_message));
    }
}

#[test]
fn partition_visibility_ddl_ack_rejects_overflowing_barrier_lsn_parts() {
    for barrier_lsn in ["100000000/0", "0/100000000"] {
        let mut request = request(complete_watermarks());
        request.barrier_lsn = barrier_lsn.to_string();

        let error =
            partition_visibility_ddl_ack_evidence(request).expect_err("invalid barrier lsn");
        assert!(error.to_string().contains("barrier_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}

#[test]
fn partition_visibility_ddl_ack_rejects_surrounding_whitespace_in_boundary_fields() {
    let cases: [RequestMutation; 5] = [
        (
            "source_id",
            |request: &mut PartitionVisibilityDdlAckRequest| {
                request.source_id = " source-a".to_string();
            },
        ),
        (
            "dataset_id",
            |request: &mut PartitionVisibilityDdlAckRequest| {
                request.dataset_id = "dataset-a ".to_string();
            },
        ),
        (
            "barrier_id",
            |request: &mut PartitionVisibilityDdlAckRequest| {
                request.barrier_id = " ddl-barrier-partitioned".to_string();
            },
        ),
        (
            "barrier_lsn",
            |request: &mut PartitionVisibilityDdlAckRequest| {
                request.barrier_lsn = " 0/16B9000".to_string();
            },
        ),
        (
            "schema_version",
            |request: &mut PartitionVisibilityDdlAckRequest| {
                request.schema_version = "schema-v2 ".to_string();
            },
        ),
    ];

    for (field, mutate) in cases {
        let mut request = request(complete_watermarks());
        mutate(&mut request);

        let error = partition_visibility_ddl_ack_evidence(request)
            .expect_err("spaced boundary field should fail");

        assert!(error.to_string().contains(field));
        assert!(error
            .to_string()
            .contains("must not contain surrounding whitespace"));
    }
}
