use trellara_checkpoint::{
    target_postgres_ddl_ack_detail_with_boundary, DdlBarrierAck, DdlBarrierStore,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlAckEvidence {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub sink: String,
    pub ack_lsn: String,
    pub barrier_lsn: Option<String>,
    pub schema_version: String,
    pub applied_statements: usize,
    pub release_gate: String,
    pub plan_sha256: String,
    pub statement_sha256s: Vec<String>,
}

impl TargetDdlAckEvidence {
    pub async fn record_barrier_ack<S: DdlBarrierStore + ?Sized>(
        self,
        store: &S,
    ) -> trellara_checkpoint::Result<()> {
        store.record_ddl_barrier_ack(self.into_barrier_ack()).await
    }

    pub fn into_barrier_ack(self) -> DdlBarrierAck {
        DdlBarrierAck {
            source_id: self.source_id,
            database_id: self.database_id,
            dataset_id: self.dataset_id,
            barrier_id: self.barrier_id,
            sink: self.sink,
            ack_lsn: self.ack_lsn,
            schema_version: self.schema_version,
            accepted: true,
            detail: target_postgres_ddl_ack_detail_with_boundary(
                self.applied_statements,
                &self.plan_sha256,
                &self.statement_sha256s,
                self.barrier_lsn.as_deref(),
            ),
        }
    }
}
