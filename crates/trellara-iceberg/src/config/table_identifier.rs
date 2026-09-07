use serde::{Deserialize, Serialize};

use crate::config::validation::validate_identifier_component;
use crate::{IcebergIntegrationError, Result};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct IcebergTableIdentifier {
    pub namespace: Vec<String>,
    pub name: String,
}

impl IcebergTableIdentifier {
    pub fn new(
        namespace: impl IntoIterator<Item = impl Into<String>>,
        name: impl Into<String>,
    ) -> Result<Self> {
        let identifier = Self {
            namespace: namespace.into_iter().map(Into::into).collect(),
            name: name.into(),
        };
        identifier.validate()?;
        Ok(identifier)
    }

    #[must_use]
    pub fn qualified_name(&self) -> String {
        let mut components = self.namespace.clone();
        components.push(self.name.clone());
        components.join(".")
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.namespace.is_empty() {
            return Err(IcebergIntegrationError::InvalidTableIdentifier {
                identifier: self.name.clone(),
                reason: "namespace cannot be empty".to_string(),
            });
        }
        for component in &self.namespace {
            validate_identifier_component(component, "namespace", &self.qualified_name())?;
        }
        validate_identifier_component(&self.name, "table name", &self.qualified_name())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergTableMapping {
    pub lake_table_name: String,
    pub target: IcebergTableIdentifier,
}

impl IcebergTableMapping {
    #[must_use]
    pub fn new(lake_table_name: impl Into<String>, target: IcebergTableIdentifier) -> Self {
        Self {
            lake_table_name: lake_table_name.into(),
            target,
        }
    }
}
