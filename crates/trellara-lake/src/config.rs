use std::collections::{BTreeMap, BTreeSet};

use trellara_protocol::RelationId;

use crate::LakeError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LakePlanConfig {
    tables: BTreeMap<(String, String), LakeTableConfig>,
}

impl LakePlanConfig {
    pub fn new(tables: Vec<LakeTableConfig>) -> Self {
        Self {
            tables: tables
                .into_iter()
                .map(|table| ((table.schema.clone(), table.table.clone()), table))
                .collect(),
        }
    }

    pub(crate) fn table_for(&self, relation: &RelationId) -> Result<&LakeTableConfig, LakeError> {
        self.tables
            .get(&(relation.schema.clone(), relation.table.clone()))
            .ok_or_else(|| LakeError::UnknownRelation {
                relation: relation.display_name(),
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LakeTableConfig {
    pub schema: String,
    pub table: String,
    pub primary_key: String,
    pub excluded_columns: BTreeSet<String>,
    pub source_schema_fingerprint: Option<u64>,
    pub partition_key_column: Option<String>,
}

impl LakeTableConfig {
    pub fn new(
        schema: impl Into<String>,
        table: impl Into<String>,
        primary_key: impl Into<String>,
    ) -> Self {
        Self {
            schema: schema.into(),
            table: table.into(),
            primary_key: primary_key.into(),
            excluded_columns: BTreeSet::new(),
            source_schema_fingerprint: None,
            partition_key_column: None,
        }
    }

    pub fn excluding(mut self, column: impl Into<String>) -> Self {
        self.excluded_columns.insert(column.into());
        self
    }

    pub fn with_source_schema_fingerprint(mut self, fingerprint: Option<u64>) -> Self {
        self.source_schema_fingerprint = fingerprint;
        self
    }

    pub fn with_partition_key_column(mut self, column: Option<String>) -> Self {
        self.partition_key_column = column;
        self
    }
}
