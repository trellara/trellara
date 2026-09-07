use serde::{Deserialize, Serialize};

use crate::IcebergTableIdentifier;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergRawCdcTableSpec {
    pub lake_table_name: String,
    pub relation: String,
    pub target: IcebergTableIdentifier,
    pub schema_fingerprint_sha256: String,
    pub columns: Vec<IcebergRawCdcColumn>,
    pub partition_fields: Vec<IcebergRawCdcPartitionField>,
    pub source_bucket_count: usize,
    pub source_buckets: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergRawCdcColumn {
    pub id: i32,
    pub name: String,
    pub column_type: IcebergRawCdcColumnType,
    pub required: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergRawCdcColumnType {
    Int,
    Long,
    String,
}

impl IcebergRawCdcColumnType {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Long => "long",
            Self::String => "string",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergRawCdcPartitionField {
    pub source_column_id: i32,
    pub source_column: String,
    pub transform: String,
}

impl IcebergRawCdcPartitionField {
    #[must_use]
    pub fn identity(source_column_id: i32, source_column: impl Into<String>) -> Self {
        Self {
            source_column_id,
            source_column: source_column.into(),
            transform: "identity".to_string(),
        }
    }
}
