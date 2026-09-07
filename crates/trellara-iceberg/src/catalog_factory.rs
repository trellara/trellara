use std::collections::HashMap;
use std::sync::Arc;

use apache_iceberg::io::{
    S3_ACCESS_KEY_ID, S3_ENDPOINT, S3_PATH_STYLE_ACCESS, S3_REGION, S3_SECRET_ACCESS_KEY,
    S3_SESSION_TOKEN,
};
use apache_iceberg::CatalogBuilder;
use apache_iceberg_rest::{RestCatalog, RestCatalogBuilder};
use apache_iceberg_storage_opendal::OpenDalStorageFactory;

use crate::{
    IcebergIntegrationError, IcebergRestCatalogConfig, IcebergS3ObjectStoreConfig, Result,
};

/// Build a REST catalog whose tables use the same S3 endpoint and credential
/// resolution as Trellara's immutable object uploader.
pub async fn build_iceberg_rest_catalog(
    catalog: &IcebergRestCatalogConfig,
    object_store: &IcebergS3ObjectStoreConfig,
) -> Result<RestCatalog> {
    catalog.validate()?;
    object_store.validate()?;
    let mut properties: HashMap<String, String> =
        catalog.catalog_properties().into_iter().collect();
    properties.extend(resolved_s3_properties(object_store)?);

    RestCatalogBuilder::default()
        .with_storage_factory(Arc::new(OpenDalStorageFactory::S3 {
            customized_credential_load: None,
        }))
        .load(&catalog.catalog_name, properties)
        .await
        .map_err(|error| IcebergIntegrationError::Catalog {
            operation: "build_rest_catalog",
            target: catalog.catalog_name.clone(),
            message: error.to_string(),
        })
}

fn resolved_s3_properties(config: &IcebergS3ObjectStoreConfig) -> Result<HashMap<String, String>> {
    let access_key = std::env::var(&config.access_key_id_env).ok();
    let secret_key = std::env::var(&config.secret_access_key_env).ok();
    let credentials = match (access_key, secret_key) {
        (Some(access_key), Some(secret_key)) => Some((access_key, secret_key)),
        (None, None) => None,
        _ => {
            return Err(IcebergIntegrationError::InvalidObjectStoreConfig {
                field: "credentials",
                reason: format!(
                    "{} and {} must either both be set or both be absent so the AWS credential chain can be used",
                    config.access_key_id_env, config.secret_access_key_env
                ),
            });
        }
    };
    let session_token = config
        .session_token_env
        .as_ref()
        .and_then(|name| std::env::var(name).ok());
    Ok(s3_properties(config, credentials, session_token))
}

fn s3_properties(
    config: &IcebergS3ObjectStoreConfig,
    credentials: Option<(String, String)>,
    session_token: Option<String>,
) -> HashMap<String, String> {
    let mut properties = HashMap::from([
        (S3_REGION.to_string(), config.region.clone()),
        (
            S3_PATH_STYLE_ACCESS.to_string(),
            config.path_style_access.to_string(),
        ),
    ]);
    if let Some(endpoint) = &config.endpoint {
        properties.insert(S3_ENDPOINT.to_string(), endpoint.clone());
    }
    if let Some((access_key, secret_key)) = credentials {
        properties.insert(S3_ACCESS_KEY_ID.to_string(), access_key);
        properties.insert(S3_SECRET_ACCESS_KEY.to_string(), secret_key);
    }
    if let Some(session_token) = session_token {
        properties.insert(S3_SESSION_TOKEN.to_string(), session_token);
    }
    properties
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_uses_official_iceberg_s3_property_names() {
        let config = IcebergS3ObjectStoreConfig::new("lake", "us-west-2")
            .with_endpoint("http://localhost:9000")
            .with_path_style_access(true);
        let properties = s3_properties(
            &config,
            Some(("access".to_string(), "secret".to_string())),
            Some("session".to_string()),
        );
        assert_eq!(properties[S3_REGION], "us-west-2");
        assert_eq!(properties[S3_ENDPOINT], "http://localhost:9000");
        assert_eq!(properties[S3_PATH_STYLE_ACCESS], "true");
        assert_eq!(properties[S3_ACCESS_KEY_ID], "access");
        assert_eq!(properties[S3_SECRET_ACCESS_KEY], "secret");
        assert_eq!(properties[S3_SESSION_TOKEN], "session");
        assert!(!properties.contains_key("access_key_id_env"));
    }
}
