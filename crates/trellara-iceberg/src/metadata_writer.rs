use std::collections::BTreeMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use trellara_lake::{
    ensure_lake_epoch_consumable, LakeEpochConsumerOptions, LakeRawCdcEpochWritePlan,
};

use crate::{
    upload_immutable_object, IcebergCompletedDataFile, IcebergFileFormat,
    IcebergImmutableUploadProof, IcebergIntegrationError, IcebergMetadataTableKind,
    IcebergMetadataTableSpec, IcebergObjectStore, IcebergS3ObjectStoreConfig, Result,
};

mod parquet;
mod rows;
mod values;
use parquet::encode_metadata_file;
use rows::metadata_rows;

#[derive(Clone, Debug)]
pub struct IcebergEncodedMetadataFile {
    pub kind: IcebergMetadataTableKind,
    pub planned_object_key: String,
    pub table_name: String,
    pub relation: String,
    pub content: Bytes,
    pub record_count: u64,
    pub checksum_rollup: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergUploadedMetadataFile {
    pub kind: IcebergMetadataTableKind,
    pub completed_file: IcebergCompletedDataFile,
    pub upload: IcebergImmutableUploadProof,
}

pub fn encode_iceberg_metadata_files(
    write_plan: &LakeRawCdcEpochWritePlan,
    specs: &[IcebergMetadataTableSpec],
    epoch_snapshot_reference: &str,
    raw_table_snapshot_ids: &BTreeMap<String, i64>,
) -> Result<Vec<IcebergEncodedMetadataFile>> {
    if epoch_snapshot_reference.trim().is_empty() || raw_table_snapshot_ids.is_empty() {
        return Err(IcebergIntegrationError::MetadataEncoding {
            table: write_plan.epoch_metadata.epochs_table.clone(),
            message: "completeness metadata requires validated raw snapshot evidence".to_string(),
        });
    }
    ensure_lake_epoch_consumable(
        &write_plan.epoch_metadata.epoch_row,
        &write_plan.epoch_metadata.verification_row,
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )?;
    let specs = specs
        .iter()
        .map(|spec| (spec.kind, spec))
        .collect::<BTreeMap<_, _>>();
    metadata_rows(write_plan, epoch_snapshot_reference, raw_table_snapshot_ids)?
        .into_iter()
        .filter(|(_, rows)| !rows.is_empty())
        .map(|(kind, rows)| {
            let spec =
                specs
                    .get(&kind)
                    .ok_or_else(|| IcebergIntegrationError::MetadataEncoding {
                        table: format!("{kind:?}"),
                        message: "metadata table specification is missing".to_string(),
                    })?;
            encode_metadata_file(write_plan, spec, rows)
        })
        .collect()
}

pub async fn upload_iceberg_metadata_file<S: IcebergObjectStore + ?Sized>(
    store: &S,
    object_store: &IcebergS3ObjectStoreConfig,
    encoded: IcebergEncodedMetadataFile,
) -> Result<IcebergUploadedMetadataFile> {
    let file_uri = object_store.object_uri(&encoded.planned_object_key);
    let upload = upload_immutable_object(
        store,
        &encoded.planned_object_key,
        &file_uri,
        encoded.content,
    )
    .await?;
    let completed_file = IcebergCompletedDataFile {
        planned_object_key: encoded.planned_object_key,
        table_name: encoded.table_name,
        relation: encoded.relation,
        source_bucket: 0,
        file_uri,
        file_format: IcebergFileFormat::Parquet,
        file_size_in_bytes: upload.content_length,
        content_sha256: upload.content_sha256.clone(),
        object_version: upload.object_version.clone(),
        record_count: encoded.record_count,
        checksum_rollup: encoded.checksum_rollup,
    };
    Ok(IcebergUploadedMetadataFile {
        kind: encoded.kind,
        completed_file,
        upload,
    })
}
