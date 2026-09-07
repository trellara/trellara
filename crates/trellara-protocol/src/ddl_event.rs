use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{default_release_gate_for_ddl, RelationId};

pub const POST_DDL_DML_RELEASE_GATE: &str = "post_ddl_dml_release";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, prost::Enumeration)]
#[repr(i32)]
pub enum DdlOperation {
    Unspecified = 0,
    AddColumn = 1,
    AddTable = 2,
    WidenColumnType = 3,
    RenameColumn = 4,
    RenameTable = 5,
    DropColumn = 6,
    AddNotNullColumn = 7,
    ChangePrimaryKey = 8,
    ChangePartitionKey = 9,
    Other = 255,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Message)]
pub struct DdlEvent {
    #[prost(string, tag = "1")]
    pub transaction_id: String,
    #[prost(uint32, tag = "2")]
    pub total_order: u32,
    #[prost(enumeration = "DdlOperation", tag = "3")]
    pub operation: i32,
    #[prost(message, optional, tag = "4")]
    pub relation: Option<RelationId>,
    #[prost(string, tag = "5")]
    pub statement: String,
    #[prost(uint64, tag = "6")]
    pub schema_fingerprint_before: u64,
    #[prost(uint64, tag = "7")]
    pub schema_fingerprint_after: u64,
    #[prost(bool, tag = "8")]
    pub target_auto_apply: bool,
    #[prost(string, tag = "9")]
    pub release_gate: String,
}

#[derive(Clone, Debug)]
pub struct DdlEventInput {
    pub transaction_id: String,
    pub total_order: u32,
    pub operation: DdlOperation,
    pub relation: RelationId,
    pub statement: String,
    pub schema_fingerprint_before: u64,
    pub schema_fingerprint_after: u64,
    pub target_auto_apply: bool,
    pub release_gate: String,
}

impl DdlEvent {
    pub fn classified(input: DdlEventInput) -> Self {
        Self {
            transaction_id: input.transaction_id,
            total_order: input.total_order,
            operation: input.operation as i32,
            relation: Some(input.relation),
            statement: input.statement,
            schema_fingerprint_before: input.schema_fingerprint_before,
            schema_fingerprint_after: input.schema_fingerprint_after,
            target_auto_apply: input.target_auto_apply,
            release_gate: input.release_gate,
        }
    }

    pub fn additive_column(
        transaction_id: impl Into<String>,
        total_order: u32,
        relation: RelationId,
        statement: impl Into<String>,
        schema_fingerprint_before: u64,
        schema_fingerprint_after: u64,
    ) -> Self {
        Self::classified(DdlEventInput {
            transaction_id: transaction_id.into(),
            total_order,
            operation: DdlOperation::AddColumn,
            relation,
            statement: statement.into(),
            schema_fingerprint_before,
            schema_fingerprint_after,
            target_auto_apply: true,
            release_gate: POST_DDL_DML_RELEASE_GATE.to_string(),
        })
    }

    pub fn manual_review(
        transaction_id: impl Into<String>,
        total_order: u32,
        operation: DdlOperation,
        relation: RelationId,
        statement: impl Into<String>,
        schema_fingerprint_before: u64,
        schema_fingerprint_after: u64,
    ) -> Self {
        Self::classified(DdlEventInput {
            transaction_id: transaction_id.into(),
            total_order,
            operation,
            relation,
            statement: statement.into(),
            schema_fingerprint_before,
            schema_fingerprint_after,
            target_auto_apply: false,
            release_gate: default_release_gate_for_ddl(operation).to_string(),
        })
    }
}
