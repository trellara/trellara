use super::*;
use apache_iceberg::Catalog;
use bytes::Bytes;

#[tokio::test]
#[ignore = "requires TRELLARA_ICEBERG_REST_URI, TRELLARA_ICEBERG_WAREHOUSE, TRELLARA_ICEBERG_S3_BUCKET, and TRELLARA_ICEBERG_S3_REGION"]
async fn live_catalog_and_immutable_s3_round_trip() {
    let (rest, object_store) = live_configs();
    rest.validate().expect("REST catalog config");
    object_store.validate().expect("S3 object store config");

    let catalog = build_iceberg_rest_catalog(&rest, &object_store)
        .await
        .expect("build REST catalog");
    catalog
        .list_namespaces(None)
        .await
        .expect("list namespaces through REST catalog");

    let store =
        OpenDalIcebergObjectStore::from_s3_config(&object_store).expect("build immutable S3 store");
    let unique = format!(
        "trellara-live-tests/immutable/{}-{}.parquet",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after Unix epoch")
            .as_nanos()
    );
    let uri = object_store.object_uri(&unique);
    let payload = Bytes::from_static(b"trellara immutable live test");
    let first = upload_immutable_object(&store, &unique, &uri, payload.clone())
        .await
        .expect("initial immutable upload");
    assert_eq!(first.status, IcebergImmutableUploadStatus::Created);
    let replay = upload_immutable_object(&store, &unique, &uri, payload)
        .await
        .expect("same-content immutable replay");
    assert_eq!(replay.status, IcebergImmutableUploadStatus::AlreadyPresent);
    let conflict = upload_immutable_object(
        &store,
        &unique,
        &uri,
        Bytes::from_static(b"different payload"),
    )
    .await
    .expect_err("different-content replay must fail");
    assert!(matches!(
        conflict,
        IcebergIntegrationError::ImmutableObjectConflict { .. }
    ));

    store
        .delete(&unique, replay.object_version.as_deref())
        .await
        .expect("remove exact live-test object version");
}

fn live_configs() -> (IcebergRestCatalogConfig, IcebergS3ObjectStoreConfig) {
    let mut rest = IcebergRestCatalogConfig::new(
        env("TRELLARA_ICEBERG_CATALOG_NAME").unwrap_or_else(|| "trellara-live".to_string()),
        env("TRELLARA_ICEBERG_REST_URI").expect("TRELLARA_ICEBERG_REST_URI"),
        env("TRELLARA_ICEBERG_WAREHOUSE").expect("TRELLARA_ICEBERG_WAREHOUSE"),
    );
    if let Some(token) = env("TRELLARA_ICEBERG_REST_TOKEN") {
        rest = rest.with_oauth_token(token);
    }
    if let Some(credential) = env("TRELLARA_ICEBERG_REST_CREDENTIAL") {
        rest = rest.with_credential(credential);
    }
    if let Some(scope) = env("TRELLARA_ICEBERG_REST_SCOPE") {
        rest = rest.with_scope(scope);
    }
    let mut object_store = IcebergS3ObjectStoreConfig::new(
        env("TRELLARA_ICEBERG_S3_BUCKET").expect("TRELLARA_ICEBERG_S3_BUCKET"),
        env("TRELLARA_ICEBERG_S3_REGION").expect("TRELLARA_ICEBERG_S3_REGION"),
    );
    if let Some(prefix) = env("TRELLARA_ICEBERG_S3_PREFIX") {
        object_store = object_store.with_key_prefix(prefix);
    }
    if let Some(endpoint) = env("TRELLARA_ICEBERG_S3_ENDPOINT") {
        object_store = object_store.with_endpoint(endpoint);
    }
    if env("TRELLARA_ICEBERG_S3_PATH_STYLE").as_deref() == Some("true") {
        object_store = object_store.with_path_style_access(true);
    }
    (rest, object_store)
}

fn env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
}
