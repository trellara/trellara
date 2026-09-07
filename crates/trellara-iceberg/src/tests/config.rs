use super::*;

#[test]
fn rest_catalog_and_s3_object_store_config_validate_and_render_properties() {
    let rest = IcebergRestCatalogConfig::new(
        "prod",
        "https://iceberg.example.test/catalog",
        "s3://warehouse",
    )
    .with_oauth_token("token")
    .with_scope("PRINCIPAL_ROLE:ALL")
    .with_property("prefix", "trellara");
    let s3 = IcebergS3ObjectStoreConfig::new("warehouse", "us-west-2")
        .with_endpoint("https://minio.example.test")
        .with_path_style_access(true);

    let config = commit_config()
        .with_rest_catalog(rest.clone())
        .expect("rest config")
        .with_s3_object_store(s3.clone())
        .expect("s3 config");

    assert_eq!(config.rest_catalog, Some(rest.clone()));
    assert_eq!(config.object_store, Some(s3.clone()));
    assert_eq!(
        rest.catalog_properties().get("uri"),
        Some(&"https://iceberg.example.test/catalog".to_string())
    );
    assert_eq!(
        rest.catalog_properties().get("token"),
        Some(&"token".to_string())
    );
    assert_eq!(
        s3.object_store_properties().get("path_style_access"),
        Some(&"true".to_string())
    );
}

#[test]
fn catalog_and_object_store_config_reject_invalid_boundaries() {
    assert!(matches!(
        IcebergRestCatalogConfig::new("prod", "s3://not-rest", "s3://warehouse").validate(),
        Err(IcebergIntegrationError::InvalidCatalogConfig { field: "uri", .. })
    ));
    assert!(matches!(
        IcebergS3ObjectStoreConfig::new(" ", "us-west-2").validate(),
        Err(IcebergIntegrationError::InvalidObjectStoreConfig {
            field: "bucket",
            ..
        })
    ));
}
