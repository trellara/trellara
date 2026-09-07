use serde::Deserialize;
use trellara_apply_postgres::ApplyTablePolicy;

use crate::default_primary_key;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TableConfig {
    pub schema: String,
    pub name: String,
    #[serde(default)]
    pub verify: Option<TableVerifyConfig>,
    #[serde(default)]
    pub contract: Option<TableContractConfig>,
}

impl TableConfig {
    pub(crate) fn relation_id(&self) -> trellara_protocol::RelationId {
        trellara_protocol::RelationId::new(0, &self.schema, &self.name)
    }

    pub(crate) fn to_apply_policy(&self) -> Option<ApplyTablePolicy> {
        let contract = self.contract.as_ref()?;
        if contract.target_owned_columns.is_empty() {
            return None;
        }
        Some(ApplyTablePolicy {
            relation: self.relation_id(),
            target_owned_columns: contract.target_owned_columns.clone(),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TableVerifyConfig {
    #[serde(default = "default_primary_key")]
    pub primary_key: String,
    #[serde(default)]
    pub excluded_columns: Vec<String>,
    #[serde(default)]
    pub row_filter: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
pub struct TableContractConfig {
    #[serde(default)]
    pub source_schema_fingerprint: Option<u64>,
    #[serde(default)]
    pub target_owned_columns: Vec<String>,
}

impl Default for TableVerifyConfig {
    fn default() -> Self {
        Self {
            primary_key: default_primary_key(),
            excluded_columns: Vec::new(),
            row_filter: None,
        }
    }
}
