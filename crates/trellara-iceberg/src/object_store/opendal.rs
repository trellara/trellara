use std::time::UNIX_EPOCH;

use async_trait::async_trait;
use bytes::Bytes;

use super::{
    IcebergObjectMetadata, IcebergObjectStore, IcebergObjectStoreError, IcebergObjectStoreErrorKind,
};
use crate::{IcebergIntegrationError, IcebergS3ObjectStoreConfig, Result};

#[derive(Clone, Debug)]
pub struct OpenDalIcebergObjectStore {
    operator: opendal::Operator,
}

impl OpenDalIcebergObjectStore {
    pub fn from_s3_config(config: &IcebergS3ObjectStoreConfig) -> Result<Self> {
        config.validate()?;
        let mut properties = vec![
            ("bucket".to_string(), config.bucket.clone()),
            ("region".to_string(), config.region.clone()),
            (
                "enable_virtual_host_style".to_string(),
                (!config.path_style_access).to_string(),
            ),
        ];
        if !config.key_prefix.is_empty() {
            properties.push(("root".to_string(), config.key_prefix.clone()));
        }
        if let Some(endpoint) = &config.endpoint {
            properties.push(("endpoint".to_string(), endpoint.clone()));
        }
        let access_key = std::env::var(&config.access_key_id_env).ok();
        let secret_key = std::env::var(&config.secret_access_key_env).ok();
        match (access_key, secret_key) {
            (Some(access_key), Some(secret_key)) => {
                properties.push(("access_key_id".to_string(), access_key));
                properties.push(("secret_access_key".to_string(), secret_key));
            }
            (None, None) => {}
            _ => return Err(IcebergIntegrationError::InvalidObjectStoreConfig {
                field: "credentials",
                reason: format!("{} and {} must either both be set or both be absent so the AWS credential chain can be used", config.access_key_id_env, config.secret_access_key_env),
            }),
        }
        if let Some(session_token_env) = &config.session_token_env {
            if let Ok(session_token) = std::env::var(session_token_env) {
                properties.push(("session_token".to_string(), session_token));
            }
        }
        let operator = opendal::Operator::from_iter::<opendal::services::S3>(properties)
            .map_err(|error| IcebergIntegrationError::ObjectStore {
                operation: "build_s3_operator",
                object_key: config.bucket.clone(),
                message: error.to_string(),
            })?
            .finish();
        if !operator.info().full_capability().write_with_if_not_exists {
            return Err(IcebergIntegrationError::InvalidObjectStoreConfig {
                field: "conditional_create",
                reason: "configured S3 backend does not advertise write-if-absent support"
                    .to_string(),
            });
        }
        Ok(Self { operator })
    }

    #[must_use]
    pub fn from_operator(operator: opendal::Operator) -> Self {
        Self { operator }
    }
}

#[async_trait]
impl IcebergObjectStore for OpenDalIcebergObjectStore {
    async fn create(
        &self,
        key: &str,
        content: Bytes,
    ) -> std::result::Result<IcebergObjectMetadata, IcebergObjectStoreError> {
        self.operator
            .write_with(key, content)
            .if_not_exists(true)
            .await
            .map_err(opendal_error)?;
        self.head(key).await
    }

    async fn read(&self, key: &str) -> std::result::Result<Bytes, IcebergObjectStoreError> {
        self.operator
            .read(key)
            .await
            .map(|buffer| buffer.to_bytes())
            .map_err(opendal_error)
    }

    async fn head(
        &self,
        key: &str,
    ) -> std::result::Result<IcebergObjectMetadata, IcebergObjectStoreError> {
        let metadata = self.operator.stat(key).await.map_err(opendal_error)?;
        Ok(object_metadata(key, &metadata))
    }

    async fn list(
        &self,
        prefix: &str,
    ) -> std::result::Result<Vec<IcebergObjectMetadata>, IcebergObjectStoreError> {
        let entries = self
            .operator
            .list_with(prefix)
            .recursive(true)
            .await
            .map_err(opendal_error)?;
        let mut objects = Vec::new();
        for entry in entries
            .into_iter()
            .filter(|entry| entry.metadata().is_file())
        {
            let metadata = self
                .operator
                .stat(entry.path())
                .await
                .map_err(opendal_error)?;
            objects.push(object_metadata(entry.path(), &metadata));
        }
        Ok(objects)
    }

    async fn delete(
        &self,
        key: &str,
        version: Option<&str>,
    ) -> std::result::Result<(), IcebergObjectStoreError> {
        let delete = self.operator.delete_with(key);
        match version {
            Some(version) => delete.version(version).await,
            None => delete.await,
        }
        .map_err(opendal_error)
    }
}

fn object_metadata(key: &str, metadata: &opendal::Metadata) -> IcebergObjectMetadata {
    let last_modified_ms = metadata.last_modified().and_then(|timestamp| {
        let system_time: std::time::SystemTime = timestamp.into();
        system_time
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| i64::try_from(duration.as_millis()).ok())
    });
    IcebergObjectMetadata {
        object_key: key.to_string(),
        content_length: metadata.content_length(),
        etag: metadata.etag().map(ToString::to_string),
        version: metadata.version().map(ToString::to_string),
        last_modified_ms,
    }
}

fn opendal_error(error: opendal::Error) -> IcebergObjectStoreError {
    let kind = match error.kind() {
        opendal::ErrorKind::NotFound => IcebergObjectStoreErrorKind::NotFound,
        opendal::ErrorKind::AlreadyExists => IcebergObjectStoreErrorKind::AlreadyExists,
        opendal::ErrorKind::ConditionNotMatch => IcebergObjectStoreErrorKind::ConditionNotMatch,
        _ => IcebergObjectStoreErrorKind::Other,
    };
    IcebergObjectStoreError::new(kind, error.to_string())
}
