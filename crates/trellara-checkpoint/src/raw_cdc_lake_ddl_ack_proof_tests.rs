use super::*;

const DIGEST: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";

#[test]
fn raw_cdc_lake_ddl_ack_detail_accepts_epoch_manifest_and_release_gate() {
    assert!(raw_cdc_lake_ddl_ack_detail_is_valid(
        &valid_detail(),
        "schema-v2"
    ));
}

#[test]
fn raw_cdc_lake_ddl_ack_detail_rejects_schema_version_mismatch() {
    assert!(!raw_cdc_lake_ddl_ack_detail_is_valid(
        &valid_detail(),
        "schema-v3"
    ));
}

#[test]
fn raw_cdc_lake_ddl_ack_detail_rejects_missing_partition_metadata() {
    assert!(!raw_cdc_lake_ddl_ack_detail_is_valid(
        &format!(
            "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; manifest_digest={DIGEST}; release_gate=post_ddl_dml_release"
        ),
        "schema-v2"
    ));
}

#[test]
fn raw_cdc_lake_ddl_ack_detail_rejects_bad_manifest_digest() {
    assert!(!raw_cdc_lake_ddl_ack_detail_is_valid(
        "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=not-a-digest; release_gate=post_ddl_dml_release",
        "schema-v2"
    ));
}

#[test]
fn raw_cdc_lake_ddl_ack_detail_rejects_spoofed_release_gate_token() {
    assert!(!raw_cdc_lake_ddl_ack_detail_is_valid(
        &format!(
            "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest={DIGEST}; previous_release_gate=post_ddl_dml_release"
        ),
        "schema-v2"
    ));
}

#[test]
fn raw_cdc_lake_ddl_ack_detail_rejects_blank_epoch() {
    assert!(!raw_cdc_lake_ddl_ack_detail_is_valid(
        &format!(
            "raw CDC lake recorded schema_version schema-v2 for epoch  in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest={DIGEST}; release_gate=post_ddl_dml_release"
        ),
        "schema-v2"
    ));
}

fn valid_detail() -> String {
    format!(
        "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest={DIGEST}; release_gate=post_ddl_dml_release"
    )
}
