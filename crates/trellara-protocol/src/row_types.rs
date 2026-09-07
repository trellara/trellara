use bytes::Bytes;
use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Message)]
pub struct RelationId {
    #[prost(uint32, tag = "1")]
    pub oid: u32,
    #[prost(string, tag = "2")]
    pub schema: String,
    #[prost(string, tag = "3")]
    pub table: String,
}

impl RelationId {
    pub fn new(oid: u32, schema: impl Into<String>, table: impl Into<String>) -> Self {
        Self {
            oid,
            schema: schema.into(),
            table: table.into(),
        }
    }

    pub fn display_name(&self) -> String {
        format!("{}.{}", self.schema, self.table)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct RelationSchemaVersion {
    #[prost(message, optional, tag = "1")]
    pub relation: Option<RelationId>,
    #[prost(uint64, tag = "2")]
    pub version: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, prost::Enumeration)]
#[repr(i32)]
pub enum ValueKind {
    Unspecified = 0,
    Null = 1,
    Text = 2,
    Binary = 3,
    UnchangedToast = 4,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct ColumnValue {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(uint32, tag = "2")]
    pub type_oid: u32,
    #[prost(enumeration = "ValueKind", tag = "3")]
    pub value_kind: i32,
    #[prost(string, tag = "4")]
    pub text_value: String,
    #[prost(bytes = "bytes", tag = "5")]
    pub binary_value: Bytes,
    #[prost(bool, tag = "6")]
    pub is_key: bool,
}

impl ColumnValue {
    pub fn null(name: impl Into<String>, type_oid: u32, is_key: bool) -> Self {
        Self {
            name: name.into(),
            type_oid,
            value_kind: ValueKind::Null as i32,
            text_value: String::new(),
            binary_value: Bytes::new(),
            is_key,
        }
    }

    pub fn text(
        name: impl Into<String>,
        type_oid: u32,
        value: impl Into<String>,
        is_key: bool,
    ) -> Self {
        Self {
            name: name.into(),
            type_oid,
            value_kind: ValueKind::Text as i32,
            text_value: value.into(),
            binary_value: Bytes::new(),
            is_key,
        }
    }

    pub fn binary(
        name: impl Into<String>,
        type_oid: u32,
        value: impl Into<Bytes>,
        is_key: bool,
    ) -> Self {
        Self {
            name: name.into(),
            type_oid,
            value_kind: ValueKind::Binary as i32,
            text_value: String::new(),
            binary_value: value.into(),
            is_key,
        }
    }

    pub fn unchanged_toast(name: impl Into<String>, type_oid: u32, is_key: bool) -> Self {
        Self {
            name: name.into(),
            type_oid,
            value_kind: ValueKind::UnchangedToast as i32,
            text_value: String::new(),
            binary_value: Bytes::new(),
            is_key,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct RowImage {
    #[prost(message, repeated, tag = "1")]
    pub columns: Vec<ColumnValue>,
}

impl RowImage {
    pub fn new(columns: Vec<ColumnValue>) -> Self {
        Self { columns }
    }
}
