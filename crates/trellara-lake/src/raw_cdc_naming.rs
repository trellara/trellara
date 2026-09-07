use crate::LakeTableConfig;

pub(crate) fn raw_cdc_table_name(dataset_id: &str, table: &LakeTableConfig) -> String {
    format!(
        "{}__raw_cdc",
        lake_table_prefix(dataset_id, &table.schema, &table.table)
    )
}

pub(crate) fn raw_cdc_object_key_hint(
    epoch_id: &str,
    table_name: &str,
    source_bucket: u32,
) -> String {
    format!(
        "{table_name}/epoch_id={}/source_bucket={source_bucket:04}/part-00000.parquet",
        object_key_segment(epoch_id)
    )
}

pub(crate) fn epoch_metadata_object_key_hint(epoch_id: &str, table_name: &str) -> String {
    format!(
        "{table_name}/epoch_id={}/part-00000.parquet",
        object_key_segment(epoch_id)
    )
}

pub(crate) fn source_bucket(source_id: &str, bucket_count: u32) -> u32 {
    stable_hash(source_id.as_bytes()) % bucket_count
}

fn stable_hash(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c9dc5_u32;
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

pub(crate) fn lake_table_prefix(dataset_id: &str, schema: &str, table: &str) -> String {
    format!(
        "{}__{}__{}",
        lake_identifier_segment(dataset_id),
        lake_identifier_segment(schema),
        lake_identifier_segment(table)
    )
}

fn lake_identifier_segment(value: &str) -> String {
    let normalized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = normalized.trim_matches('_');
    if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed.to_string()
    }
}

fn object_key_segment(value: &str) -> String {
    let normalized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '=') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = normalized.trim_matches('_');
    if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed.to_string()
    }
}
