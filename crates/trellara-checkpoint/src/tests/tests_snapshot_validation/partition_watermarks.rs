use super::*;

#[test]
fn partition_checkpoint_requires_flow_identity_and_some_watermark() {
    validate_partition_checkpoint(&partition_checkpoint(0, "0/16B9000", ""))
        .expect("partial partition checkpoint update can advance durable LSN");
    validate_partition_checkpoint(&partition_checkpoint(0, "", "0/16B8800"))
        .expect("partial partition checkpoint update can advance applied LSN");

    let error = validate_partition_checkpoint(&partition_checkpoint(0, "", ""))
        .expect_err("missing watermarks rejected");
    assert!(error
        .to_string()
        .contains("without a durable or applied LSN"));

    let mut checkpoint = partition_checkpoint(0, "0/16B9000", "0/16B8800");
    checkpoint.dataset_id = " ".to_string();
    let error = validate_partition_checkpoint(&checkpoint).expect_err("empty dataset rejected");
    assert!(error.to_string().contains("dataset_id must not be empty"));
}

#[test]
fn partition_checkpoint_rejects_invalid_partial_lsn_evidence() {
    let error = validate_partition_checkpoint(&partition_checkpoint(0, "bad-lsn", ""))
        .expect_err("invalid durable LSN rejected");
    assert!(error.to_string().contains("last_durable_lsn"));
    assert!(error.to_string().contains("non-zero PostgreSQL LSN"));

    let error = validate_partition_checkpoint(&partition_checkpoint(0, "", "0/0"))
        .expect_err("zero applied LSN rejected");
    assert!(error.to_string().contains("last_applied_lsn"));
    assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
}

#[test]
fn partition_checkpoint_rejects_applied_lsn_ahead_of_durable_lsn() {
    let error = validate_partition_checkpoint(&partition_checkpoint(0, "0/16B8800", "0/16B9000"))
        .expect_err("applied ahead of durable rejected");

    assert!(error
        .to_string()
        .contains("applied LSN 0/16B9000 is ahead of durable LSN 0/16B8800"));
}

#[test]
fn partition_watermark_summary_rejects_missing_partition_lsn_evidence() {
    let error = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        2,
        vec![
            partition_checkpoint(0, "0/16B9000", "0/16B8800"),
            partition_checkpoint(1, "0/16B9000", ""),
        ],
    )
    .expect_err("missing partition applied lsn rejected");

    assert!(error
        .to_string()
        .contains("missing durable or applied LSN evidence"));
}
