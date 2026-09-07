use serde::{Deserialize, Serialize};

use crate::config::{IcebergRestCatalogConfig, IcebergS3ObjectStoreConfig, IcebergTableMapping};
use crate::Result;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergCommitConfig {
    pub table_mappings: Vec<IcebergTableMapping>,
    pub accept_complete_with_gaps: bool,
    pub rest_catalog: Option<IcebergRestCatalogConfig>,
    pub object_store: Option<IcebergS3ObjectStoreConfig>,
}

impl IcebergCommitConfig {
    #[must_use]
    pub fn new(table_mappings: Vec<IcebergTableMapping>) -> Self {
        Self {
            table_mappings,
            accept_complete_with_gaps: false,
            rest_catalog: None,
            object_store: None,
        }
    }

    #[must_use]
    pub fn accepting_complete_with_gaps(mut self) -> Self {
        self.accept_complete_with_gaps = true;
        self
    }

    pub fn with_rest_catalog(mut self, rest_catalog: IcebergRestCatalogConfig) -> Result<Self> {
        rest_catalog.validate()?;
        self.rest_catalog = Some(rest_catalog);
        Ok(self)
    }

    pub fn with_s3_object_store(
        mut self,
        object_store: IcebergS3ObjectStoreConfig,
    ) -> Result<Self> {
        object_store.validate()?;
        self.object_store = Some(object_store);
        Ok(self)
    }
}
