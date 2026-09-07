use std::collections::BTreeSet;

use crate::raw_cdc::metadata::RawCdcMetadataInput;
use crate::LakeError;

use super::fields::{validate_clean_metadata_field, validate_lsn_window};

pub(super) fn validate(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    validate_input_identity(input)?;
    validate_source_rows(input)?;
    validate_table_rows(input)?;
    validate_partition_rows(input)
}

fn validate_source_rows(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    let mut seen_sources = BTreeSet::new();
    for source in &input.source_rows {
        validate_clean_metadata_field("source_rows[].epoch_id", &source.epoch_id)?;
        validate_clean_metadata_field("source_rows[].source_id", &source.source_id)?;
        validate_clean_metadata_field("source_rows[].start_lsn", &source.start_lsn)?;
        validate_clean_metadata_field("source_rows[].end_lsn", &source.end_lsn)?;
        validate_lsn_window(
            "source_rows[].start_lsn",
            &source.start_lsn,
            "source_rows[].end_lsn",
            &source.end_lsn,
        )?;
        validate_metadata_epoch_boundary(
            "source",
            &source.source_id,
            &source.epoch_id,
            &input.epoch_id,
        )?;
        if !seen_sources.insert(source.source_id.clone()) {
            return Err(LakeError::DuplicateRawCdcEpochSource {
                epoch_id: input.epoch_id.clone(),
                source_id: source.source_id.clone(),
            });
        }
    }
    Ok(())
}

fn validate_table_rows(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    let mut seen_tables = BTreeSet::new();
    for table in &input.table_rows {
        validate_clean_metadata_field("table_rows[].epoch_id", &table.epoch_id)?;
        validate_clean_metadata_field("table_rows[].relation", &table.relation)?;
        validate_metadata_epoch_boundary(
            "table",
            &table.relation,
            &table.epoch_id,
            &input.epoch_id,
        )?;
        if !seen_tables.insert(table.relation.clone()) {
            return Err(LakeError::DuplicateRawCdcEpochTable {
                epoch_id: input.epoch_id.clone(),
                relation: table.relation.clone(),
            });
        }
    }
    Ok(())
}

fn validate_partition_rows(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    let mut seen_partitions = BTreeSet::new();
    let source_ids = input
        .source_rows
        .iter()
        .map(|source| source.source_id.as_str())
        .collect::<BTreeSet<_>>();
    for partition in &input.partition_rows {
        validate_clean_metadata_field("partition_rows[].epoch_id", &partition.epoch_id)?;
        validate_clean_metadata_field("partition_rows[].source_id", &partition.source_id)?;
        validate_clean_metadata_field(
            "partition_rows[].first_commit_lsn",
            &partition.first_commit_lsn,
        )?;
        validate_clean_metadata_field(
            "partition_rows[].last_commit_lsn",
            &partition.last_commit_lsn,
        )?;
        validate_lsn_window(
            "partition_rows[].first_commit_lsn",
            &partition.first_commit_lsn,
            "partition_rows[].last_commit_lsn",
            &partition.last_commit_lsn,
        )?;
        validate_metadata_epoch_boundary(
            "partition",
            &format!("{}:{}", partition.source_id, partition.partition_id),
            &partition.epoch_id,
            &input.epoch_id,
        )?;
        validate_partition_source_evidence(&source_ids, &partition.source_id)?;
        if !seen_partitions.insert((partition.source_id.clone(), partition.partition_id)) {
            return Err(LakeError::DuplicateRawCdcEpochPartition {
                epoch_id: input.epoch_id.clone(),
                source_id: partition.source_id.clone(),
                partition_id: partition.partition_id,
            });
        }
    }
    Ok(())
}

fn validate_partition_source_evidence(
    source_ids: &BTreeSet<&str>,
    source_id: &str,
) -> Result<(), LakeError> {
    if source_ids.contains(source_id) {
        return Ok(());
    }
    Err(LakeError::InvalidRawCdcEpochMetadataField {
        field: "partition_rows[].source_id",
        reason: format!("partition source {source_id} must have matching source row evidence"),
    })
}

fn validate_input_identity(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    validate_clean_metadata_field("dataset_id", &input.dataset_id)?;
    validate_clean_metadata_field("epoch_id", &input.epoch_id)?;
    for source_id in &input.required_sources {
        validate_clean_metadata_field("required_sources[]", source_id)?;
    }
    Ok(())
}

fn validate_metadata_epoch_boundary(
    row_kind: &'static str,
    row_id: &str,
    row_epoch_id: &str,
    epoch_id: &str,
) -> Result<(), LakeError> {
    if row_epoch_id == epoch_id {
        Ok(())
    } else {
        Err(LakeError::RawCdcEpochMetadataBoundaryMismatch {
            row_kind,
            row_id: row_id.to_string(),
            row_epoch_id: row_epoch_id.to_string(),
            epoch_id: epoch_id.to_string(),
        })
    }
}
