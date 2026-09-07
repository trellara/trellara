use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::Mutex;

use super::*;

#[derive(Default)]
pub(super) struct FaultInjectingStore {
    objects: Mutex<BTreeMap<String, Bytes>>,
    fail_after_create_once: AtomicBool,
}

impl FaultInjectingStore {
    fn failing_after_create() -> Self {
        Self {
            fail_after_create_once: AtomicBool::new(true),
            ..Self::default()
        }
    }
}

#[async_trait]
impl IcebergObjectStore for FaultInjectingStore {
    async fn create(
        &self,
        object_key: &str,
        content: Bytes,
    ) -> std::result::Result<IcebergObjectMetadata, IcebergObjectStoreError> {
        let mut objects = self.objects.lock().await;
        if objects.contains_key(object_key) {
            return Err(IcebergObjectStoreError::new(
                IcebergObjectStoreErrorKind::AlreadyExists,
                "conditional create rejected an existing key",
            ));
        }
        let content_length = content.len() as u64;
        objects.insert(object_key.to_string(), content);
        if self.fail_after_create_once.swap(false, Ordering::SeqCst) {
            return Err(IcebergObjectStoreError::new(
                IcebergObjectStoreErrorKind::Other,
                "simulated response loss after durable create",
            ));
        }
        Ok(metadata(object_key, content_length))
    }

    async fn read(&self, object_key: &str) -> std::result::Result<Bytes, IcebergObjectStoreError> {
        self.objects
            .lock()
            .await
            .get(object_key)
            .cloned()
            .ok_or_else(not_found)
    }

    async fn head(
        &self,
        object_key: &str,
    ) -> std::result::Result<IcebergObjectMetadata, IcebergObjectStoreError> {
        let content_length = self
            .objects
            .lock()
            .await
            .get(object_key)
            .map(Bytes::len)
            .ok_or_else(not_found)? as u64;
        Ok(metadata(object_key, content_length))
    }

    async fn list(
        &self,
        prefix: &str,
    ) -> std::result::Result<Vec<IcebergObjectMetadata>, IcebergObjectStoreError> {
        Ok(self
            .objects
            .lock()
            .await
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(key, value)| metadata(key, value.len() as u64))
            .collect())
    }

    async fn delete(
        &self,
        object_key: &str,
        _version: Option<&str>,
    ) -> std::result::Result<(), IcebergObjectStoreError> {
        self.objects.lock().await.remove(object_key);
        Ok(())
    }
}

#[tokio::test]
async fn immutable_upload_reconciles_lost_create_response() {
    let store = FaultInjectingStore::failing_after_create();
    let proof = upload_immutable_object(
        &store,
        "raw/epoch-1/file.parquet",
        "s3://lake/raw/epoch-1/file.parquet",
        Bytes::from_static(b"parquet bytes"),
    )
    .await
    .expect("reconciled upload");

    assert_eq!(
        proof.status,
        IcebergImmutableUploadStatus::ReconciledAfterAmbiguousFailure
    );
    assert_eq!(proof.content_length, 13);
}

#[tokio::test]
async fn concurrent_identical_uploads_create_exactly_one_immutable_object() {
    let store = std::sync::Arc::new(FaultInjectingStore::default());
    let mut tasks = Vec::new();
    for _ in 0..32 {
        let store = store.clone();
        tasks.push(tokio::spawn(async move {
            upload_immutable_object(
                store.as_ref(),
                "raw/epoch-1/file.parquet",
                "s3://lake/raw/epoch-1/file.parquet",
                Bytes::from_static(b"same content"),
            )
            .await
        }));
    }

    let mut created = 0;
    let mut already_present = 0;
    for task in tasks {
        match task.await.expect("task").expect("upload").status {
            IcebergImmutableUploadStatus::Created => created += 1,
            IcebergImmutableUploadStatus::AlreadyPresent => already_present += 1,
            IcebergImmutableUploadStatus::ReconciledAfterAmbiguousFailure => {}
        }
    }
    assert_eq!(created, 1);
    assert_eq!(already_present, 31);
    assert_eq!(store.objects.lock().await.len(), 1);
}

#[tokio::test]
async fn immutable_upload_rejects_conflicting_replay() {
    let store = FaultInjectingStore::default();
    upload_immutable_object(
        &store,
        "raw/epoch-1/file.parquet",
        "s3://lake/raw/epoch-1/file.parquet",
        Bytes::from_static(b"first"),
    )
    .await
    .expect("first upload");

    let error = upload_immutable_object(
        &store,
        "raw/epoch-1/file.parquet",
        "s3://lake/raw/epoch-1/file.parquet",
        Bytes::from_static(b"different"),
    )
    .await
    .expect_err("conflicting replay");
    assert!(matches!(
        error,
        IcebergIntegrationError::ImmutableObjectConflict { .. }
    ));
}

fn metadata(object_key: &str, content_length: u64) -> IcebergObjectMetadata {
    IcebergObjectMetadata {
        object_key: object_key.to_string(),
        content_length,
        etag: Some(format!("etag-{content_length}")),
        version: Some("v1".to_string()),
        last_modified_ms: Some(1),
    }
}

fn not_found() -> IcebergObjectStoreError {
    IcebergObjectStoreError::new(IcebergObjectStoreErrorKind::NotFound, "not found")
}
