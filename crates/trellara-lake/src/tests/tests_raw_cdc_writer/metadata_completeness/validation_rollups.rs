use super::*;

#[test]
fn metadata_plan_rejects_partition_event_rollup_mismatch() {
    let mut input = input();
    let mut partition = partition_row("store-001", 0);
    partition.event_count = 1;
    input.source_rows = vec![source_row("store-001")];
    input.partition_rows = vec![partition];

    let error = build_epoch_metadata(input).expect_err("partition event rollup rejected");

    assert!(matches!(
        error,
        LakeError::RawCdcEpochPartitionRollupMismatch {
            epoch_id,
            field: "event_count",
            expected: 2,
            actual: 1,
        } if epoch_id == "epoch-1"
    ));
}

#[test]
fn metadata_plan_rejects_invalid_source_lsn_window_before_manifest_digest() {
    for (field, start_lsn, end_lsn, expected_reason) in [
        (
            "source_rows[].start_lsn",
            "not-a-lsn",
            "0/16B0100",
            "must be a non-zero PostgreSQL LSN",
        ),
        (
            "source_rows[].end_lsn",
            "0/16B0000",
            "0/0",
            "must be a non-zero PostgreSQL LSN",
        ),
        (
            "source_rows[].start_lsn",
            "0/16B0100",
            "0/16B0000",
            "must be before or equal to source_rows[].end_lsn",
        ),
    ] {
        let mut input = input();
        let mut source = source_row("store-001");
        source.start_lsn = start_lsn.to_string();
        source.end_lsn = end_lsn.to_string();
        input.source_rows = vec![source];

        let error = build_epoch_metadata(input).expect_err("invalid source LSN rejected");

        assert!(matches!(
            error,
            LakeError::InvalidRawCdcEpochMetadataField { field: actual, reason }
                if actual == field && reason.contains(expected_reason)
        ));
    }
}

#[test]
fn metadata_plan_rejects_source_transaction_rollup_mismatch() {
    let mut input = input();
    let mut source = source_row("store-001");
    source.transaction_count = 2;
    input.source_rows = vec![source];

    let error = build_epoch_metadata(input).expect_err("source transaction rollup rejected");

    assert!(matches!(
        error,
        LakeError::RawCdcEpochSourceRollupMismatch {
            epoch_id,
            field: "transaction_count",
            expected: 1,
            actual: 2,
        } if epoch_id == "epoch-1"
    ));
}

#[test]
fn metadata_plan_rejects_source_change_rollup_mismatch() {
    let mut input = input();
    let mut source = source_row("store-001");
    source.change_count = 3;
    input.source_rows = vec![source];

    let error = build_epoch_metadata(input).expect_err("source change rollup rejected");

    assert!(matches!(
        error,
        LakeError::RawCdcEpochSourceRollupMismatch {
            epoch_id,
            field: "change_count",
            expected: 2,
            actual: 3,
        } if epoch_id == "epoch-1"
    ));
}

#[test]
fn metadata_plan_rejects_source_checksum_rollup_mismatch() {
    let mut input = input();
    let mut source = source_row("store-001");
    source.checksum_rollup = 4;
    input.source_rows = vec![source];

    let error = build_epoch_metadata(input).expect_err("source checksum rollup rejected");

    assert!(matches!(
        error,
        LakeError::RawCdcEpochSourceRollupMismatch {
            epoch_id,
            field: "checksum_rollup",
            expected: 3,
            actual: 4,
        } if epoch_id == "epoch-1"
    ));
}

#[test]
fn metadata_plan_rejects_table_change_rollup_mismatch() {
    let mut input = input();
    let mut table = table_row("public.sales", "epoch-1");
    table.change_count = 3;
    input.table_rows = vec![table];

    let error = build_epoch_metadata(input).expect_err("table change rollup rejected");

    assert!(matches!(
        error,
        LakeError::RawCdcEpochTableRollupMismatch {
            epoch_id,
            field: "change_count",
            expected: 2,
            actual: 3,
        } if epoch_id == "epoch-1"
    ));
}
