use super::*;

#[test]
fn partition_watermark_rejects_out_of_range_partitions() {
    let error = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        2,
        vec![partition_checkpoint(2, "0/16B8000", "0/16B7800")],
    )
    .expect_err("out of range");

    assert!(error
        .to_string()
        .contains("partition 2 is outside expected range 0..1"));
}

#[test]
fn partition_watermark_rejects_cross_flow_checkpoint_evidence() {
    let mut wrong_source = partition_checkpoint(0, "0/16B8000", "0/16B7800");
    wrong_source.source_id = "source-b".to_string();

    let error =
        PartitionWatermarkSummary::from_checkpoints("source-a", "sales", 1, vec![wrong_source])
            .expect_err("wrong source");

    assert!(error
        .to_string()
        .contains("partition checkpoint 0 belongs to source-b.sales, expected source-a.sales"));

    let mut wrong_dataset = partition_checkpoint(0, "0/16B8000", "0/16B7800");
    wrong_dataset.dataset_id = "orders".to_string();

    let error =
        PartitionWatermarkSummary::from_checkpoints("source-a", "sales", 1, vec![wrong_dataset])
            .expect_err("wrong dataset");

    assert!(error
        .to_string()
        .contains("partition checkpoint 0 belongs to source-a.orders, expected source-a.sales"));
}

#[test]
fn partition_watermark_rejects_invalid_lsn_evidence() {
    let error = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        1,
        vec![partition_checkpoint(0, "not-a-lsn", "0/16B7800")],
    )
    .expect_err("invalid durable lsn");

    assert!(error.to_string().contains(
        "partition checkpoint 0 last_durable_lsn not-a-lsn must be a non-zero PostgreSQL LSN"
    ));

    let error = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        1,
        vec![partition_checkpoint(0, "0/16B8000", "0/0")],
    )
    .expect_err("zero applied lsn");

    assert!(error
        .to_string()
        .contains("partition checkpoint 0 last_applied_lsn 0/0 must be a non-zero PostgreSQL LSN"));

    let error = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        1,
        vec![partition_checkpoint(0, "0/100000000", "0/16B7800")],
    )
    .expect_err("overflowing durable lsn rejected");
    assert!(error
        .to_string()
        .contains("last_durable_lsn 0/100000000 must be a non-zero PostgreSQL LSN"));
}

#[test]
fn partition_watermark_rejects_lsn_evidence_with_surrounding_whitespace() {
    let error = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        1,
        vec![partition_checkpoint(0, " 0/16B8000", "0/16B7800")],
    )
    .expect_err("spaced durable lsn");

    assert!(error.to_string().contains(
        "partition checkpoint 0 last_durable_lsn must not contain surrounding whitespace"
    ));

    let error = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        1,
        vec![partition_checkpoint(0, "0/16B8000", "0/16B7800 ")],
    )
    .expect_err("spaced applied lsn");

    assert!(error.to_string().contains(
        "partition checkpoint 0 last_applied_lsn must not contain surrounding whitespace"
    ));
}

#[test]
fn partition_watermark_rejects_applied_lsn_ahead_of_durable_lsn() {
    let error = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        1,
        vec![partition_checkpoint(0, "0/16B7800", "0/16B8000")],
    )
    .expect_err("applied ahead of durable");

    assert!(error.to_string().contains(
        "partition checkpoint 0 applied LSN 0/16B8000 is ahead of durable LSN 0/16B7800"
    ));
}

#[test]
fn observed_partition_count_rejects_usize_overflow() {
    let error = crate::partition::observed_partition_count(u32::MAX as usize + 1)
        .expect_err("partition count overflow");

    assert!(error
        .to_string()
        .contains("observed partition count 4294967296 exceeds supported u32 range"));
}
