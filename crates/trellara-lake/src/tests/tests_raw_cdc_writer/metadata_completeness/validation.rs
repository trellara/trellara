use super::*;

#[test]
fn metadata_plan_rejects_observed_gap_source_without_lag_reason() {
    for state in [
        LakeEpochSourceState::Lagging,
        LakeEpochSourceState::Missing,
        LakeEpochSourceState::Quarantined,
        LakeEpochSourceState::Reseeding,
    ] {
        let mut input = input();
        let mut source = source_row("store-001");
        source.state = state;
        source.lag_reason = None;
        input.source_rows = vec![source];

        let error = build_epoch_metadata(input).expect_err("missing lag reason rejected");

        assert!(matches!(
            error,
            LakeError::InvalidRawCdcEpochMetadataField {
                field: "source_rows[].lag_reason",
                reason,
            } if reason.contains("requires explicit lag reason evidence")
        ));
    }
}

#[test]
fn metadata_plan_rejects_unclean_gap_source_lag_reason() {
    for lag_reason in ["", " quarantined source"] {
        let mut input = input();
        let mut source = source_row("store-001");
        source.state = LakeEpochSourceState::Quarantined;
        source.lag_reason = Some(lag_reason.to_string());
        input.source_rows = vec![source];

        let error = build_epoch_metadata(input).expect_err("unclean lag reason rejected");

        assert!(matches!(
            error,
            LakeError::InvalidRawCdcEpochMetadataField {
                field: "source_rows[].lag_reason",
                ..
            }
        ));
    }
}

#[test]
fn metadata_plan_rejects_duplicate_source_rows_before_counting_completeness() {
    let mut input = input();
    input.source_rows = vec![source_row("store-001"), source_row("store-001")];

    let error = build_epoch_metadata(input).expect_err("duplicate source rejected");

    assert!(matches!(
        error,
        LakeError::DuplicateRawCdcEpochSource {
            epoch_id,
            source_id,
        } if epoch_id == "epoch-1" && source_id == "store-001"
    ));
}

#[test]
fn metadata_plan_rejects_source_rows_from_another_epoch() {
    let mut input = input();
    let mut source = source_row("store-001");
    source.epoch_id = "epoch-other".to_string();
    input.source_rows = vec![source];

    let error = build_epoch_metadata(input).expect_err("source boundary rejected");

    assert!(matches!(
        error,
        LakeError::RawCdcEpochMetadataBoundaryMismatch {
            row_kind: "source",
            row_id,
            row_epoch_id,
            epoch_id,
        } if row_id == "store-001" && row_epoch_id == "epoch-other" && epoch_id == "epoch-1"
    ));
}

#[test]
fn metadata_plan_rejects_table_rows_from_another_epoch() {
    let mut input = input();
    input.table_rows = vec![table_row("public.sales", "epoch-other")];

    let error = build_epoch_metadata(input).expect_err("table boundary rejected");

    assert!(matches!(
        error,
        LakeError::RawCdcEpochMetadataBoundaryMismatch {
            row_kind: "table",
            row_id,
            row_epoch_id,
            epoch_id,
        } if row_id == "public.sales" && row_epoch_id == "epoch-other" && epoch_id == "epoch-1"
    ));
}

#[test]
fn metadata_plan_rejects_padded_epoch_metadata_identity_before_manifest_digest() {
    for field in [
        "dataset_id",
        "epoch_id",
        "required_sources[]",
        "source_rows[].epoch_id",
        "source_rows[].source_id",
        "source_rows[].start_lsn",
        "source_rows[].end_lsn",
        "table_rows[].epoch_id",
        "table_rows[].relation",
        "partition_rows[].epoch_id",
        "partition_rows[].source_id",
        "partition_rows[].first_commit_lsn",
        "partition_rows[].last_commit_lsn",
    ] {
        let mut input = input();
        input.required_sources = required_sources();
        input.source_rows = vec![source_row("store-001")];
        input.table_rows = vec![table_row("public.sales", "epoch-1")];
        input.partition_rows = vec![partition_row("store-001", 0)];
        match field {
            "dataset_id" => input.dataset_id = " retail".to_string(),
            "epoch_id" => input.epoch_id = "epoch-1 ".to_string(),
            "required_sources[]" => {
                input.required_sources.insert(" store-003".to_string());
            }
            "source_rows[].epoch_id" => input.source_rows[0].epoch_id = " epoch-1".to_string(),
            "source_rows[].source_id" => input.source_rows[0].source_id = "store-001 ".to_string(),
            "source_rows[].start_lsn" => input.source_rows[0].start_lsn = " 0/16B0000".to_string(),
            "source_rows[].end_lsn" => input.source_rows[0].end_lsn = "0/16B0100 ".to_string(),
            "table_rows[].epoch_id" => input.table_rows[0].epoch_id = " epoch-1".to_string(),
            "table_rows[].relation" => input.table_rows[0].relation = "public.sales ".to_string(),
            "partition_rows[].epoch_id" => {
                input.partition_rows[0].epoch_id = " epoch-1".to_string();
            }
            "partition_rows[].source_id" => {
                input.partition_rows[0].source_id = "store-001 ".to_string();
            }
            "partition_rows[].first_commit_lsn" => {
                input.partition_rows[0].first_commit_lsn = " 0/16B0000".to_string();
            }
            "partition_rows[].last_commit_lsn" => {
                input.partition_rows[0].last_commit_lsn = "0/16B0100 ".to_string();
            }
            _ => unreachable!("test fields are exhaustive"),
        }

        let error = build_epoch_metadata(input).expect_err("padded metadata identity rejected");

        assert!(matches!(
            error,
            LakeError::InvalidRawCdcEpochMetadataField { field: actual, reason }
                if actual == field && reason == "must not contain surrounding whitespace"
        ));
    }
}

#[test]
fn metadata_plan_rejects_duplicate_partition_rows_before_manifest_digest() {
    let mut input = input();
    input.source_rows = vec![source_row("store-001")];
    input.partition_rows = vec![partition_row("store-001", 0), partition_row("store-001", 0)];

    let error = build_epoch_metadata(input).expect_err("duplicate partition rejected");

    assert!(matches!(
        error,
        LakeError::DuplicateRawCdcEpochPartition {
            epoch_id,
            source_id,
            partition_id: 0,
        } if epoch_id == "epoch-1" && source_id == "store-001"
    ));
}

#[test]
fn metadata_plan_rejects_partition_rows_without_source_evidence() {
    let mut input = input();
    input.partition_rows = vec![partition_row("store-001", 0)];

    let error = build_epoch_metadata(input).expect_err("orphan partition source rejected");

    assert!(matches!(
        error,
        LakeError::InvalidRawCdcEpochMetadataField {
            field: "partition_rows[].source_id",
            reason,
        } if reason.contains("matching source row evidence")
            && reason.contains("store-001")
    ));
}

#[test]
fn metadata_plan_rejects_partition_rows_for_unlisted_source() {
    let mut input = input();
    input.source_rows = vec![source_row("store-002")];
    input.partition_rows = vec![partition_row("store-001", 0)];

    let error = build_epoch_metadata(input).expect_err("wrong partition source rejected");

    assert!(matches!(
        error,
        LakeError::InvalidRawCdcEpochMetadataField {
            field: "partition_rows[].source_id",
            reason,
        } if reason.contains("matching source row evidence")
            && reason.contains("store-001")
    ));
}

#[test]
fn metadata_plan_rejects_partition_rows_from_another_epoch() {
    let mut input = input();
    let mut partition = partition_row("store-001", 7);
    partition.epoch_id = "epoch-other".to_string();
    input.partition_rows = vec![partition];

    let error = build_epoch_metadata(input).expect_err("partition boundary rejected");

    assert!(matches!(
        error,
        LakeError::RawCdcEpochMetadataBoundaryMismatch {
            row_kind: "partition",
            row_id,
            row_epoch_id,
            epoch_id,
        } if row_id == "store-001:7" && row_epoch_id == "epoch-other" && epoch_id == "epoch-1"
    ));
}

#[test]
fn metadata_plan_rejects_invalid_partition_lsn_window_before_manifest_digest() {
    for (field, first_lsn, last_lsn, expected_reason) in [
        (
            "partition_rows[].first_commit_lsn",
            "not-a-lsn",
            "0/16B0100",
            "must be a non-zero PostgreSQL LSN",
        ),
        (
            "partition_rows[].last_commit_lsn",
            "0/16B0000",
            "0/0",
            "must be a non-zero PostgreSQL LSN",
        ),
        (
            "partition_rows[].first_commit_lsn",
            "0/16B0100",
            "0/16B0000",
            "must be before or equal to partition_rows[].last_commit_lsn",
        ),
    ] {
        let mut input = input();
        let mut partition = partition_row("store-001", 0);
        partition.first_commit_lsn = first_lsn.to_string();
        partition.last_commit_lsn = last_lsn.to_string();
        input.partition_rows = vec![partition];

        let error = build_epoch_metadata(input).expect_err("invalid partition LSN rejected");

        assert!(matches!(
            error,
            LakeError::InvalidRawCdcEpochMetadataField { field: actual, reason }
                if actual == field && reason.contains(expected_reason)
        ));
    }
}

#[test]
fn metadata_plan_rejects_duplicate_table_rows_before_manifest_digest() {
    let mut input = input();
    input.table_rows = vec![
        table_row("public.sales", "epoch-1"),
        table_row("public.sales", "epoch-1"),
    ];

    let error = build_epoch_metadata(input).expect_err("duplicate table rejected");

    assert!(matches!(
        error,
        LakeError::DuplicateRawCdcEpochTable {
            epoch_id,
            relation,
        } if epoch_id == "epoch-1" && relation == "public.sales"
    ));
}
