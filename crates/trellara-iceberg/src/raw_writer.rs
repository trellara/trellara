use bytes::Bytes;
use serde::{Deserialize, Serialize};
use trellara_lake::{write_raw_cdc_parquet_file, LakeRawCdcDataFilePlan, LakeRawCdcEpochWritePlan};

use crate::{
    upload_immutable_object, IcebergCompletedDataFile, IcebergImmutableUploadProof,
    IcebergIntegrationError, IcebergObjectStore, IcebergS3ObjectStoreConfig, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergUploadedRawDataFile {
    pub completed_file: IcebergCompletedDataFile,
    pub upload: IcebergImmutableUploadProof,
}

pub async fn write_and_upload_raw_cdc_file<S: IcebergObjectStore + ?Sized>(
    store: &S,
    object_store: &IcebergS3ObjectStoreConfig,
    write_plan: &LakeRawCdcEpochWritePlan,
    data_file: &LakeRawCdcDataFilePlan,
) -> Result<IcebergUploadedRawDataFile> {
    object_store.validate()?;
    let file_uri = object_store.object_uri(&data_file.object_key_hint);
    let encoded = write_raw_cdc_parquet_file(Vec::new(), write_plan, data_file, &file_uri, None)?;
    let upload = upload_immutable_object(
        store,
        &data_file.object_key_hint,
        &file_uri,
        Bytes::from(encoded.writer),
    )
    .await?;
    if encoded.evidence.file_size_in_bytes != upload.content_length
        || encoded.evidence.content_sha256 != upload.content_sha256
    {
        return Err(IcebergIntegrationError::CompletedDataFileMismatch {
            planned_object_key: data_file.object_key_hint.clone(),
            field: "immutable_upload_proof",
            expected: format!(
                "{}:{}",
                encoded.evidence.file_size_in_bytes, encoded.evidence.content_sha256
            ),
            actual: format!("{}:{}", upload.content_length, upload.content_sha256),
        });
    }
    let mut evidence = encoded.evidence;
    evidence.object_version = upload.object_version.clone();
    let completed_file = IcebergCompletedDataFile::try_from(evidence)?;
    Ok(IcebergUploadedRawDataFile {
        completed_file,
        upload,
    })
}
