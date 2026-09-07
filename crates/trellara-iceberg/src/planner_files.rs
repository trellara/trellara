use std::collections::BTreeMap;

use trellara_lake::LakeRawCdcDataFilePlan;

use crate::{IcebergCompletedDataFile, IcebergFileFormat, IcebergIntegrationError, Result};

pub(crate) fn completed_files_by_key(
    completed_files: Vec<IcebergCompletedDataFile>,
) -> Result<BTreeMap<String, IcebergCompletedDataFile>> {
    let mut by_key = BTreeMap::new();
    for file in completed_files {
        validate_completed_file_shape(&file)?;
        let key = file.planned_object_key.clone();
        if by_key.insert(key.clone(), file).is_some() {
            return Err(IcebergIntegrationError::DuplicateCompletedDataFile {
                planned_object_key: key,
            });
        }
    }
    Ok(by_key)
}

pub(crate) fn validate_completed_file(
    planned: &LakeRawCdcDataFilePlan,
    completed: &IcebergCompletedDataFile,
) -> Result<()> {
    compare_file_field(
        completed,
        "table_name",
        &planned.table_name,
        &completed.table_name,
    )?;
    compare_file_field(
        completed,
        "relation",
        &planned.relation,
        &completed.relation,
    )?;
    compare_file_field(
        completed,
        "source_bucket",
        &planned.source_bucket.to_string(),
        &completed.source_bucket.to_string(),
    )?;
    compare_file_field(
        completed,
        "checksum_rollup",
        &planned.checksum_rollup.to_string(),
        &completed.checksum_rollup.to_string(),
    )?;
    let planned_record_count = u64::try_from(planned.change_count).map_err(|_| {
        IcebergIntegrationError::CountOverflow {
            field: "planned_file.change_count",
        }
    })?;
    compare_file_field(
        completed,
        "record_count",
        &planned_record_count.to_string(),
        &completed.record_count.to_string(),
    )
}

pub(crate) fn validate_completed_file_shape(file: &IcebergCompletedDataFile) -> Result<()> {
    if file.planned_object_key.trim().is_empty() {
        return invalid_file(file, "planned_object_key", "cannot be blank");
    }
    if file.file_size_in_bytes == 0 {
        return invalid_file(file, "file_size_in_bytes", "must be greater than zero");
    }
    if !valid_sha256(&file.content_sha256) {
        return invalid_file(
            file,
            "content_sha256",
            "must be a lowercase 64-character SHA-256 digest",
        );
    }
    if file
        .object_version
        .as_ref()
        .is_some_and(|version| version.trim().is_empty())
    {
        return invalid_file(file, "object_version", "cannot be blank when present");
    }
    if file.record_count == 0 {
        return invalid_file(file, "record_count", "must be greater than zero");
    }
    if !valid_uri(&file.file_uri) {
        return invalid_file(
            file,
            "file_uri",
            "must be an absolute URI with a non-empty scheme and location",
        );
    }
    if !file.file_uri.to_ascii_lowercase().ends_with(".parquet") {
        return invalid_file(file, "file_uri", "must identify a Parquet file");
    }
    if file.file_format != IcebergFileFormat::Parquet {
        return invalid_file(file, "file_format", "only Parquet is supported");
    }
    Ok(())
}

fn compare_file_field(
    file: &IcebergCompletedDataFile,
    field: &'static str,
    expected: &str,
    actual: &str,
) -> Result<()> {
    if expected == actual {
        return Ok(());
    }
    Err(IcebergIntegrationError::CompletedDataFileMismatch {
        planned_object_key: file.planned_object_key.clone(),
        field,
        expected: expected.to_string(),
        actual: actual.to_string(),
    })
}

fn invalid_file<T>(
    file: &IcebergCompletedDataFile,
    field: &'static str,
    reason: &str,
) -> Result<T> {
    Err(IcebergIntegrationError::InvalidCompletedDataFile {
        planned_object_key: file.planned_object_key.clone(),
        field,
        reason: reason.to_string(),
    })
}

fn valid_uri(uri: &str) -> bool {
    let Some((scheme, location)) = uri.split_once("://") else {
        return false;
    };
    !scheme.is_empty()
        && !location.is_empty()
        && scheme.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
        })
        && !uri.chars().any(char::is_control)
}

fn valid_sha256(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
