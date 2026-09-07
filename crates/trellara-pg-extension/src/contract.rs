use serde::Serialize;
use std::fmt;

use crate::{supports_postgres_major, HANDOFF_CONTRACT, SOURCE_ACKNOWLEDGEMENT_CONTRACT};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeCaptureContract {
    pub source_id: String,
    pub dataset_id: String,
    pub protocol_version: u32,
    pub postgres_major: u16,
    pub transaction_visibility: &'static str,
    pub delivery_semantics: &'static str,
    pub handoff: &'static str,
    pub source_acknowledgement: &'static str,
    pub broker_io_inside_postgres: bool,
    pub data_plane_ready: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CaptureContractError {
    BlankSourceId,
    BlankDatasetId,
    PaddedSourceId,
    PaddedDatasetId,
    UnsupportedPostgresMajor(u16),
}

impl fmt::Display for CaptureContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSourceId => formatter.write_str("source_id cannot be blank"),
            Self::BlankDatasetId => formatter.write_str("dataset_id cannot be blank"),
            Self::PaddedSourceId => formatter.write_str("source_id cannot be padded"),
            Self::PaddedDatasetId => formatter.write_str("dataset_id cannot be padded"),
            Self::UnsupportedPostgresMajor(major) => write!(
                formatter,
                "PostgreSQL {major} is unsupported; supported majors are 15, 16, 17, and 18"
            ),
        }
    }
}

impl std::error::Error for CaptureContractError {}

pub fn native_capture_contract(
    source_id: &str,
    dataset_id: &str,
    postgres_major: u16,
) -> Result<NativeCaptureContract, CaptureContractError> {
    let source_id = validate_identity(source_id, IdentityField::SourceId)?;
    let dataset_id = validate_identity(dataset_id, IdentityField::DatasetId)?;

    if !supports_postgres_major(postgres_major) {
        return Err(CaptureContractError::UnsupportedPostgresMajor(
            postgres_major,
        ));
    }

    Ok(NativeCaptureContract {
        source_id: source_id.to_owned(),
        dataset_id: dataset_id.to_owned(),
        protocol_version: trellara_protocol::PROTOCOL_VERSION,
        postgres_major,
        transaction_visibility: "committed_transactions_only",
        delivery_semantics: "at_least_once_with_idempotent_apply",
        handoff: HANDOFF_CONTRACT,
        source_acknowledgement: SOURCE_ACKNOWLEDGEMENT_CONTRACT,
        broker_io_inside_postgres: false,
        data_plane_ready: crate::runtime_data_plane_ready(),
    })
}

enum IdentityField {
    SourceId,
    DatasetId,
}

fn validate_identity(value: &str, field: IdentityField) -> Result<&str, CaptureContractError> {
    let trimmed = value.trim();
    match field {
        IdentityField::SourceId if trimmed.is_empty() => Err(CaptureContractError::BlankSourceId),
        IdentityField::DatasetId if trimmed.is_empty() => Err(CaptureContractError::BlankDatasetId),
        IdentityField::SourceId if trimmed != value => Err(CaptureContractError::PaddedSourceId),
        IdentityField::DatasetId if trimmed != value => Err(CaptureContractError::PaddedDatasetId),
        _ => Ok(value),
    }
}
