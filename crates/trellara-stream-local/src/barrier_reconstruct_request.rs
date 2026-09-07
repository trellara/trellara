use std::collections::BTreeSet;

use crate::{LocalStreamError, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalBarrierReconstructionRequest {
    pub source_id: String,
    pub dataset_id: String,
    pub manifest_offset: i64,
    pub commit_offset: i64,
    pub partition_offsets: Vec<LocalPartitionChunkOffset>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LocalPartitionChunkOffset {
    pub partition_id: u32,
    pub offset: i64,
}

pub(crate) fn validate_barrier_reconstruction_request(
    request: &LocalBarrierReconstructionRequest,
) -> Result<()> {
    validate_clean_request_field("source_id", &request.source_id)?;
    validate_clean_request_field("dataset_id", &request.dataset_id)?;
    validate_offset("manifest_offset", request.manifest_offset)?;
    validate_offset("commit_offset", request.commit_offset)?;
    let mut seen = BTreeSet::new();
    for offset in &request.partition_offsets {
        validate_offset("partition_offsets[].offset", offset.offset)?;
        if !seen.insert(offset.partition_id) {
            return Err(LocalStreamError::DuplicateBarrierPartitionOffset {
                partition_id: offset.partition_id,
            });
        }
    }
    Ok(())
}

fn validate_clean_request_field(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(LocalStreamError::InvalidBarrierReconstructionField {
            field,
            reason: "must not be empty".to_string(),
        });
    }
    if value != value.trim() {
        return Err(LocalStreamError::InvalidBarrierReconstructionField {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    Ok(())
}

fn validate_offset(field: &'static str, offset: i64) -> Result<()> {
    if offset < 0 {
        return Err(LocalStreamError::NegativeBarrierOffset { field, offset });
    }
    Ok(())
}
