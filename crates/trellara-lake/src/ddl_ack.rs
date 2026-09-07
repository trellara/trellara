use trellara_checkpoint::{DdlBarrierAck, DdlBarrierStore};

use crate::{
    ddl_ack_validation::{validate_ack_lsn, validate_clean_field, validate_sha256_digest},
    LakeError,
};

const RAW_CDC_LAKE_SINK: &str = "raw_cdc_lake";
const RELEASE_GATE: &str = "post_ddl_dml_release";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawCdcLakeDdlAckRequest {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub ack_lsn: String,
    pub schema_version: String,
    pub epoch_id: String,
    pub metadata_table: String,
    pub partition_metadata_table: String,
    pub manifest_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawCdcLakeDdlAckEvidence {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub sink: String,
    pub ack_lsn: String,
    pub schema_version: String,
    pub epoch_id: String,
    pub metadata_table: String,
    pub partition_metadata_table: String,
    pub manifest_digest: String,
    pub release_gate: String,
}

pub fn raw_cdc_lake_ddl_ack_evidence(
    request: RawCdcLakeDdlAckRequest,
) -> Result<RawCdcLakeDdlAckEvidence, LakeError> {
    let evidence = RawCdcLakeDdlAckEvidence {
        source_id: request.source_id,
        database_id: request.database_id,
        dataset_id: request.dataset_id,
        barrier_id: request.barrier_id,
        sink: RAW_CDC_LAKE_SINK.to_string(),
        ack_lsn: request.ack_lsn,
        schema_version: request.schema_version,
        epoch_id: request.epoch_id,
        metadata_table: request.metadata_table,
        partition_metadata_table: request.partition_metadata_table,
        manifest_digest: request.manifest_digest,
        release_gate: RELEASE_GATE.to_string(),
    };
    evidence.validate()?;
    Ok(evidence)
}

impl RawCdcLakeDdlAckEvidence {
    pub async fn record_barrier_ack<S: DdlBarrierStore + ?Sized>(
        self,
        store: &S,
    ) -> trellara_checkpoint::Result<()> {
        store.record_ddl_barrier_ack(self.into_barrier_ack()).await
    }

    pub fn into_barrier_ack(self) -> DdlBarrierAck {
        let detail = format!(
            "raw CDC lake recorded schema_version {} for epoch {} in metadata table {}; partition_metadata_table={}; manifest_digest={}; release_gate={}",
            self.schema_version,
            self.epoch_id,
            self.metadata_table,
            self.partition_metadata_table,
            self.manifest_digest,
            self.release_gate
        );
        DdlBarrierAck {
            source_id: self.source_id,
            database_id: self.database_id,
            dataset_id: self.dataset_id,
            barrier_id: self.barrier_id,
            sink: self.sink,
            ack_lsn: self.ack_lsn,
            schema_version: self.schema_version,
            accepted: true,
            detail,
        }
    }

    fn validate(&self) -> Result<(), LakeError> {
        validate_clean_field("source_id", &self.source_id)?;
        validate_clean_field("database_id", &self.database_id)?;
        validate_clean_field("dataset_id", &self.dataset_id)?;
        validate_clean_field("barrier_id", &self.barrier_id)?;
        validate_ack_lsn(&self.ack_lsn)?;
        validate_clean_field("schema_version", &self.schema_version)?;
        validate_clean_field("epoch_id", &self.epoch_id)?;
        validate_clean_field("metadata_table", &self.metadata_table)?;
        validate_clean_field("partition_metadata_table", &self.partition_metadata_table)?;
        validate_sha256_digest("manifest_digest", &self.manifest_digest)?;
        Ok(())
    }
}
