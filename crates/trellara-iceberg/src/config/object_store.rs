use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::validation::{
    object_store_config_error, validate_http_uri, validate_non_blank, validate_optional_non_blank,
};
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergS3ObjectStoreConfig {
    pub bucket: String,
    pub region: String,
    #[serde(default)]
    pub key_prefix: String,
    pub endpoint: Option<String>,
    pub access_key_id_env: String,
    pub secret_access_key_env: String,
    pub session_token_env: Option<String>,
    pub path_style_access: bool,
}

impl IcebergS3ObjectStoreConfig {
    #[must_use]
    pub fn new(bucket: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            region: region.into(),
            key_prefix: String::new(),
            endpoint: None,
            access_key_id_env: "AWS_ACCESS_KEY_ID".to_string(),
            secret_access_key_env: "AWS_SECRET_ACCESS_KEY".to_string(),
            session_token_env: Some("AWS_SESSION_TOKEN".to_string()),
            path_style_access: false,
        }
    }

    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    #[must_use]
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    #[must_use]
    pub fn with_path_style_access(mut self, enabled: bool) -> Self {
        self.path_style_access = enabled;
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_non_blank("bucket", &self.bucket, object_store_config_error)?;
        validate_non_blank("region", &self.region, object_store_config_error)?;
        validate_key_prefix(&self.key_prefix)?;
        validate_non_blank(
            "access_key_id_env",
            &self.access_key_id_env,
            object_store_config_error,
        )?;
        validate_non_blank(
            "secret_access_key_env",
            &self.secret_access_key_env,
            object_store_config_error,
        )?;
        validate_optional_non_blank(
            "session_token_env",
            self.session_token_env.as_deref(),
            object_store_config_error,
        )?;
        if let Some(endpoint) = &self.endpoint {
            validate_http_uri("endpoint", endpoint, object_store_config_error)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn object_store_properties(&self) -> BTreeMap<String, String> {
        let mut properties = BTreeMap::from([
            ("bucket".to_string(), self.bucket.clone()),
            ("region".to_string(), self.region.clone()),
            ("key_prefix".to_string(), self.key_prefix.clone()),
            (
                "access_key_id_env".to_string(),
                self.access_key_id_env.clone(),
            ),
            (
                "secret_access_key_env".to_string(),
                self.secret_access_key_env.clone(),
            ),
            (
                "path_style_access".to_string(),
                self.path_style_access.to_string(),
            ),
        ]);
        if let Some(endpoint) = &self.endpoint {
            properties.insert("endpoint".to_string(), endpoint.clone());
        }
        if let Some(session_token_env) = &self.session_token_env {
            properties.insert("session_token_env".to_string(), session_token_env.clone());
        }
        properties
    }

    #[must_use]
    pub fn object_uri(&self, object_key: &str) -> String {
        let prefix = self.key_prefix.trim_matches('/');
        let key = object_key.trim_start_matches('/');
        if prefix.is_empty() {
            format!("s3://{}/{key}", self.bucket)
        } else {
            format!("s3://{}/{prefix}/{key}", self.bucket)
        }
    }
}

fn validate_key_prefix(prefix: &str) -> Result<()> {
    if prefix.starts_with('/') || prefix.ends_with('/') {
        return Err(object_store_config_error(
            "key_prefix",
            "must not start or end with '/'".to_string(),
        ));
    }
    if prefix
        .split('/')
        .any(|segment| segment == "." || segment == "..")
    {
        return Err(object_store_config_error(
            "key_prefix",
            "must not contain '.' or '..' path segments".to_string(),
        ));
    }
    Ok(())
}
