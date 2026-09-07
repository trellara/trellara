use trellara_protocol::{
    default_release_gate_for_ddl, default_target_auto_apply_for_ddl, DdlOperation, Operation,
    RelationId, ReplicaIdentity, RowImage,
};

#[derive(Clone, Debug, PartialEq)]
pub enum LogicalEvent {
    Begin {
        transaction_id: String,
        begin_lsn: String,
    },
    Change {
        transaction_id: Option<String>,
        relation: RelationId,
        operation: Operation,
        replica_identity: ReplicaIdentity,
        before: Option<RowImage>,
        after: Option<RowImage>,
    },
    RelationMetadata {
        relation: RelationId,
        schema_fingerprint: u64,
    },
    Truncate {
        transaction_id: Option<String>,
        relations: Vec<RelationId>,
    },
    Ddl {
        transaction_id: Option<String>,
        operation: DdlOperation,
        relation: RelationId,
        statement: String,
        schema_fingerprint_before: u64,
        schema_fingerprint_after: u64,
        target_auto_apply: bool,
        release_gate: String,
    },
    Commit {
        commit_lsn: String,
        commit_timestamp_ms: i64,
    },
    StreamStart {
        transaction_id: String,
        first_segment: bool,
    },
    StreamStop,
    StreamCommit {
        transaction_id: String,
        commit_lsn: String,
        commit_timestamp_ms: i64,
    },
    StreamAbort {
        transaction_id: String,
        subtransaction_id: String,
    },
    Abort,
}

impl LogicalEvent {
    pub fn observed_ddl(input: ObservedDdlEvent) -> Self {
        let target_auto_apply = default_target_auto_apply_for_ddl(input.operation);
        let release_gate = default_release_gate_for_ddl(input.operation).to_string();
        Self::Ddl {
            transaction_id: input.transaction_id,
            operation: input.operation,
            relation: input.relation,
            statement: input.statement,
            schema_fingerprint_before: input.schema_fingerprint_before,
            schema_fingerprint_after: input.schema_fingerprint_after,
            target_auto_apply,
            release_gate,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObservedDdlEvent {
    pub transaction_id: Option<String>,
    pub operation: DdlOperation,
    pub relation: RelationId,
    pub statement: String,
    pub schema_fingerprint_before: u64,
    pub schema_fingerprint_after: u64,
}
