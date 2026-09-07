use serde::Serialize;
use trellara_protocol::{RelationId, ReplicaIdentity};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedRelation {
    pub id: RelationId,
    pub replica_identity: ReplicaIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TablePreflight {
    pub schema: String,
    pub name: String,
    pub exists: bool,
    pub replica_identity: Option<ReplicaIdentity>,
    pub primary_key_columns: Vec<String>,
    pub columns: Vec<TableColumn>,
    pub schema_fingerprint: Option<u64>,
    pub update_delete_safe: bool,
    pub issues: Vec<String>,
    pub contract_notes: Vec<String>,
}

impl TablePreflight {
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TableColumn {
    pub ordinal_position: i32,
    pub name: String,
    pub type_oid: u32,
    pub type_name: String,
    pub nullable: bool,
    pub is_key: bool,
}
