#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlAckContext {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub ack_lsn: String,
    pub barrier_lsn: Option<String>,
    pub schema_version: String,
}

impl TargetDdlAckContext {
    pub fn new(
        source_id: impl Into<String>,
        database_id: impl Into<String>,
        dataset_id: impl Into<String>,
        ack_lsn: impl Into<String>,
        schema_version: impl Into<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            database_id: database_id.into(),
            dataset_id: dataset_id.into(),
            ack_lsn: ack_lsn.into(),
            barrier_lsn: None,
            schema_version: schema_version.into(),
        }
    }

    pub fn with_barrier_lsn(mut self, barrier_lsn: impl Into<String>) -> Self {
        self.barrier_lsn = Some(barrier_lsn.into());
        self
    }

    pub fn from_envelope(
        envelope: &trellara_protocol::TransactionEnvelope,
        schema_version: impl Into<String>,
    ) -> Self {
        Self::new(
            &envelope.source_id,
            &envelope.database_id,
            &envelope.dataset_id,
            &envelope.commit_lsn,
            schema_version,
        )
        .with_barrier_lsn(&envelope.commit_lsn)
    }
}
