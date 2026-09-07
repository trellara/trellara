pub use trellara_protocol::POST_DDL_DML_RELEASE_GATE as RAW_CDC_LAKE_DDL_RELEASE_GATE;

const RAW_CDC_LAKE_PREFIX: &str = "raw CDC lake recorded schema_version ";

#[derive(Debug, Eq, PartialEq)]
struct RawCdcLakeDetail<'a> {
    schema_version: &'a str,
    epoch_id: &'a str,
    metadata_table: &'a str,
}

pub fn raw_cdc_lake_ddl_ack_detail_is_valid(detail: &str, required_schema_version: &str) -> bool {
    let mut recorded_detail = None;
    let mut partition_metadata_table = None;
    let mut manifest_digest = None;
    let mut release_gate_matches = false;

    for token in detail.split(';').map(str::trim) {
        if let Some(parsed) = parse_recorded_detail(token) {
            recorded_detail = Some(parsed);
        }
        if let Some(table) = token.strip_prefix("partition_metadata_table=") {
            partition_metadata_table = Some(table);
        }
        if let Some(digest) = token.strip_prefix("manifest_digest=") {
            manifest_digest = Some(digest);
        }
        if token
            .strip_prefix("release_gate=")
            .is_some_and(|release_gate| release_gate == RAW_CDC_LAKE_DDL_RELEASE_GATE)
        {
            release_gate_matches = true;
        }
    }

    recorded_detail.is_some_and(|detail| {
        detail.schema_version == required_schema_version
            && clean_token(detail.epoch_id)
            && clean_token(detail.metadata_table)
    }) && partition_metadata_table.is_some_and(clean_token)
        && manifest_digest.is_some_and(sha256_is_valid)
        && release_gate_matches
}

fn parse_recorded_detail(token: &str) -> Option<RawCdcLakeDetail<'_>> {
    let token = token.strip_prefix(RAW_CDC_LAKE_PREFIX)?;
    let (schema_version, token) = token.split_once(" for epoch ")?;
    let (epoch_id, metadata_table) = token.split_once(" in metadata table ")?;

    Some(RawCdcLakeDetail {
        schema_version,
        epoch_id,
        metadata_table,
    })
}

fn clean_token(value: &str) -> bool {
    !value.is_empty() && value.trim() == value
}

fn sha256_is_valid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
#[path = "raw_cdc_lake_ddl_ack_proof_tests.rs"]
mod tests;
