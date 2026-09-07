use std::fmt;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::{IcebergIntegrationError, Result};

#[cfg(feature = "production-writer")]
mod opendal;
mod verification;
#[cfg(feature = "production-writer")]
pub use opendal::OpenDalIcebergObjectStore;
use verification::{sha256, validate_object_key, verify_content};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IcebergObjectStoreErrorKind {
    NotFound,
    AlreadyExists,
    ConditionNotMatch,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcebergObjectStoreError {
    pub kind: IcebergObjectStoreErrorKind,
    pub message: String,
}

impl IcebergObjectStoreError {
    #[must_use]
    pub fn new(kind: IcebergObjectStoreErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for IcebergObjectStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for IcebergObjectStoreError {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergObjectMetadata {
    pub object_key: String,
    pub content_length: u64,
    pub etag: Option<String>,
    pub version: Option<String>,
    pub last_modified_ms: Option<i64>,
}

#[async_trait]
pub trait IcebergObjectStore: Send + Sync {
    async fn create(
        &self,
        object_key: &str,
        content: Bytes,
    ) -> std::result::Result<IcebergObjectMetadata, IcebergObjectStoreError>;
    async fn read(&self, object_key: &str) -> std::result::Result<Bytes, IcebergObjectStoreError>;
    async fn head(
        &self,
        object_key: &str,
    ) -> std::result::Result<IcebergObjectMetadata, IcebergObjectStoreError>;
    async fn list(
        &self,
        prefix: &str,
    ) -> std::result::Result<Vec<IcebergObjectMetadata>, IcebergObjectStoreError>;
    async fn delete(
        &self,
        object_key: &str,
        version: Option<&str>,
    ) -> std::result::Result<(), IcebergObjectStoreError>;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergImmutableUploadStatus {
    Created,
    AlreadyPresent,
    ReconciledAfterAmbiguousFailure,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergImmutableUploadProof {
    pub object_key: String,
    pub file_uri: String,
    pub content_length: u64,
    pub content_sha256: String,
    pub etag: Option<String>,
    pub object_version: Option<String>,
    pub status: IcebergImmutableUploadStatus,
}

pub async fn upload_immutable_object<S: IcebergObjectStore + ?Sized>(
    store: &S,
    object_key: &str,
    file_uri: impl Into<String>,
    content: Bytes,
) -> Result<IcebergImmutableUploadProof> {
    validate_object_key(object_key)?;
    let expected_size =
        u64::try_from(content.len()).map_err(|_| IcebergIntegrationError::CountOverflow {
            field: "object_content_length",
        })?;
    let expected_sha256 = sha256(&content);
    let status = match store.create(object_key, content).await {
        Ok(_) => IcebergImmutableUploadStatus::Created,
        Err(error)
            if matches!(
                error.kind,
                IcebergObjectStoreErrorKind::AlreadyExists
                    | IcebergObjectStoreErrorKind::ConditionNotMatch
            ) =>
        {
            IcebergImmutableUploadStatus::AlreadyPresent
        }
        Err(original) => match verify_content(store, object_key, expected_size, &expected_sha256).await {
            Ok(_) => IcebergImmutableUploadStatus::ReconciledAfterAmbiguousFailure,
            Err(conflict @ IcebergIntegrationError::ImmutableObjectConflict { .. }) => return Err(conflict),
            Err(reconciliation_error) => return Err(IcebergIntegrationError::ObjectStore {
                operation: "create_and_reconcile",
                object_key: object_key.to_string(),
                message: format!("create failed: {original}; reconciliation did not prove the object: {reconciliation_error}"),
            }),
        },
    };
    let metadata = verify_content(store, object_key, expected_size, &expected_sha256).await?;
    Ok(IcebergImmutableUploadProof {
        object_key: object_key.to_string(),
        file_uri: file_uri.into(),
        content_length: metadata.content_length,
        content_sha256: expected_sha256,
        etag: metadata.etag,
        object_version: metadata.version,
        status,
    })
}
