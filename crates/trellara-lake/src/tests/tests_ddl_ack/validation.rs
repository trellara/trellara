use super::*;

#[test]
fn raw_cdc_lake_ddl_ack_rejects_missing_fields() {
    for case in [
        ack_case(
            "source_id",
            "",
            "dataset-a",
            "epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "dataset_id",
            "source-a",
            " ",
            "epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "epoch_id",
            "source-a",
            "dataset-a",
            "",
            "_epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "manifest_digest",
            "source-a",
            "dataset-a",
            "epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            "",
        ),
        ack_case(
            "partition_metadata_table",
            "source-a",
            "dataset-a",
            "epoch-a",
            "_epochs",
            "",
            MANIFEST_DIGEST,
        ),
    ] {
        let error = build_ack(case).expect_err("missing field");
        assert!(matches!(error, LakeError::MissingDdlAckField { field } if field == case.field));
    }
}

#[test]
fn raw_cdc_lake_ddl_ack_rejects_invalid_lsn() {
    for ack_lsn in [
        "not-a-lsn",
        "0/0",
        "xyz/16B9000",
        "100000000/0",
        "0/100000000",
    ] {
        let error = raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "dataset-a".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            ack_lsn: ack_lsn.to_string(),
            schema_version: "schema-v2".to_string(),
            epoch_id: "epoch-a".to_string(),
            metadata_table: "_epochs".to_string(),
            partition_metadata_table: "_trellara_epoch_partitions".to_string(),
            manifest_digest: MANIFEST_DIGEST.to_string(),
        })
        .expect_err("invalid lsn");
        assert!(matches!(error, LakeError::InvalidDdlAckLsn { .. }));
    }
}

#[test]
fn raw_cdc_lake_ddl_ack_rejects_schema_version_with_surrounding_whitespace() {
    let error = raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "dataset-a".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: " schema-v2 ".to_string(),
        epoch_id: "epoch-a".to_string(),
        metadata_table: "_epochs".to_string(),
        partition_metadata_table: "_trellara_epoch_partitions".to_string(),
        manifest_digest: MANIFEST_DIGEST.to_string(),
    })
    .expect_err("spaced schema version");

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
fn raw_cdc_lake_ddl_ack_rejects_fields_with_surrounding_whitespace() {
    for case in [
        ack_case(
            "source_id",
            " source-a",
            "dataset-a",
            "epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "dataset_id",
            "source-a",
            "dataset-a ",
            "epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "barrier_id",
            "source-a",
            "dataset-a",
            "epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "ack_lsn",
            "source-a",
            "dataset-a",
            "epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "epoch_id",
            "source-a",
            "dataset-a",
            " epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "metadata_table",
            "source-a",
            "dataset-a",
            "epoch-a",
            " _epochs",
            "_trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "partition_metadata_table",
            "source-a",
            "dataset-a",
            "epoch-a",
            "_epochs",
            " _trellara_epoch_partitions",
            MANIFEST_DIGEST,
        ),
        ack_case(
            "manifest_digest",
            "source-a",
            "dataset-a",
            "epoch-a",
            "_epochs",
            "_trellara_epoch_partitions",
            " abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ),
    ] {
        let error = build_ack(case).expect_err("spaced field");
        assert!(matches!(
            error,
            LakeError::InvalidDdlAckField { field, reason }
                if field == case.field && reason == "must not contain surrounding whitespace"
        ));
    }
}

#[test]
fn raw_cdc_lake_ddl_ack_rejects_malformed_manifest_digest() {
    for manifest_digest in ["not-a-digest", &"g".repeat(64), &"a".repeat(63)] {
        let error = raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "dataset-a".to_string(),
            barrier_id: "ddl-barrier-123".to_string(),
            ack_lsn: "0/16B9000".to_string(),
            schema_version: "schema-v2".to_string(),
            epoch_id: "epoch-a".to_string(),
            metadata_table: "_epochs".to_string(),
            partition_metadata_table: "_trellara_epoch_partitions".to_string(),
            manifest_digest: manifest_digest.to_string(),
        })
        .expect_err("malformed manifest digest");

        assert!(matches!(
            error,
            LakeError::InvalidDdlAckField {
                field: "manifest_digest",
                ..
            }
        ));
    }
}

#[derive(Clone, Copy)]
struct AckCase<'a> {
    field: &'static str,
    source_id: &'a str,
    dataset_id: &'a str,
    barrier_id: &'a str,
    ack_lsn: &'a str,
    epoch_id: &'a str,
    table: &'a str,
    partition_table: &'a str,
    manifest_digest: &'a str,
}

fn ack_case<'a>(
    field: &'static str,
    source_id: &'a str,
    dataset_id: &'a str,
    epoch_id: &'a str,
    table: &'a str,
    partition_table: &'a str,
    manifest_digest: &'a str,
) -> AckCase<'a> {
    AckCase {
        field,
        source_id,
        dataset_id,
        barrier_id: if field == "barrier_id" {
            " ddl-barrier-123"
        } else {
            "ddl-barrier-123"
        },
        ack_lsn: if field == "ack_lsn" {
            " 0/16B9000"
        } else {
            "0/16B9000"
        },
        epoch_id,
        table,
        partition_table,
        manifest_digest,
    }
}

fn build_ack(case: AckCase<'_>) -> Result<RawCdcLakeDdlAckEvidence, LakeError> {
    raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
        source_id: case.source_id.to_string(),
        database_id: "retail".to_string(),
        dataset_id: case.dataset_id.to_string(),
        barrier_id: case.barrier_id.to_string(),
        ack_lsn: case.ack_lsn.to_string(),
        schema_version: "schema-v2".to_string(),
        epoch_id: case.epoch_id.to_string(),
        metadata_table: case.table.to_string(),
        partition_metadata_table: case.partition_table.to_string(),
        manifest_digest: case.manifest_digest.to_string(),
    })
}
