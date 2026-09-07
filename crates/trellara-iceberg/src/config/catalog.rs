use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::validation::{
    catalog_config_error, validate_http_uri, validate_non_blank, validate_optional_non_blank,
};
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergRestCatalogConfig {
    pub catalog_name: String,
    pub uri: String,
    pub warehouse: String,
    pub credential: Option<String>,
    pub token: Option<String>,
    pub scope: Option<String>,
    pub extra_properties: BTreeMap<String, String>,
}

impl IcebergRestCatalogConfig {
    #[must_use]
    pub fn new(
        catalog_name: impl Into<String>,
        uri: impl Into<String>,
        warehouse: impl Into<String>,
    ) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            uri: uri.into(),
            warehouse: warehouse.into(),
            credential: None,
            token: None,
            scope: None,
            extra_properties: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_oauth_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    #[must_use]
    pub fn with_credential(mut self, credential: impl Into<String>) -> Self {
        self.credential = Some(credential.into());
        self
    }

    #[must_use]
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    #[must_use]
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_properties.insert(key.into(), value.into());
        self
    }

    pub fn validate(&self) -> Result<()> {
        validate_non_blank("catalog_name", &self.catalog_name, catalog_config_error)?;
        validate_http_uri("uri", &self.uri, catalog_config_error)?;
        validate_non_blank("warehouse", &self.warehouse, catalog_config_error)?;
        validate_optional_non_blank(
            "credential",
            self.credential.as_deref(),
            catalog_config_error,
        )?;
        validate_optional_non_blank("token", self.token.as_deref(), catalog_config_error)?;
        validate_optional_non_blank("scope", self.scope.as_deref(), catalog_config_error)?;
        for (key, value) in &self.extra_properties {
            validate_non_blank("extra_properties.key", key, catalog_config_error)?;
            validate_non_blank("extra_properties.value", value, catalog_config_error)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn catalog_properties(&self) -> BTreeMap<String, String> {
        let mut properties = self.extra_properties.clone();
        properties.insert("uri".to_string(), self.uri.clone());
        properties.insert("warehouse".to_string(), self.warehouse.clone());
        if let Some(credential) = &self.credential {
            properties.insert("credential".to_string(), credential.clone());
        }
        if let Some(token) = &self.token {
            properties.insert("token".to_string(), token.clone());
        }
        if let Some(scope) = &self.scope {
            properties.insert("scope".to_string(), scope.clone());
        }
        properties
    }
}
