use sha2::{Digest, Sha256};

use super::{IcebergObjectMetadata, IcebergObjectStore, IcebergObjectStoreError};
use crate::{IcebergIntegrationError, Result};

pub(super) async fn verify_content<S: IcebergObjectStore + ?Sized>(
    store: &S,
    object_key: &str,
    expected_size: u64,
    expected_sha256: &str,
) -> Result<IcebergObjectMetadata> {
    let content = store
        .read(object_key)
        .await
        .map_err(|error| object_store_error("read_after_create", object_key, error))?;
    let actual_sha256 = sha256(&content);
    if actual_sha256 != expected_sha256 {
        return Err(IcebergIntegrationError::ImmutableObjectConflict {
            object_key: object_key.to_string(),
            expected_sha256: expected_sha256.to_string(),
            actual_sha256,
        });
    }
    let metadata = store
        .head(object_key)
        .await
        .map_err(|error| object_store_error("head_after_create", object_key, error))?;
    if metadata.content_length != expected_size {
        return Err(IcebergIntegrationError::ObjectStore {
            operation: "verify_content_length",
            object_key: object_key.to_string(),
            message: format!(
                "expected {expected_size} bytes after immutable create, found {}",
                metadata.content_length
            ),
        });
    }
    Ok(metadata)
}

pub(super) fn validate_object_key(object_key: &str) -> Result<()> {
    if object_key.is_empty()
        || object_key.starts_with('/')
        || object_key.ends_with('/')
        || object_key.trim() != object_key
        || object_key
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(IcebergIntegrationError::ObjectStore {
            operation: "validate_object_key",
            object_key: object_key.to_string(),
            message: "key must be relative, normalized, non-empty, and contain no empty, '.' or '..' segments".to_string(),
        });
    }
    Ok(())
}

pub(super) fn sha256(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn object_store_error(
    operation: &'static str,
    object_key: &str,
    error: IcebergObjectStoreError,
) -> IcebergIntegrationError {
    IcebergIntegrationError::ObjectStore {
        operation,
        object_key: object_key.to_string(),
        message: error.to_string(),
    }
}
