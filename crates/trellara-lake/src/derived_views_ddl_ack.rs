use trellara_checkpoint::{DdlBarrierAck, DdlBarrierStore};

use crate::{
    ddl_ack_validation::{validate_ack_lsn, validate_clean_field, validate_sha256_digest},
    LakeError,
};

const SPARK_DERIVED_VIEWS_SINK: &str = "spark_derived_views";
const RELEASE_GATE: &str = "post_ddl_dml_release";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparkDerivedViewsDdlAckRequest {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub ack_lsn: String,
    pub schema_version: String,
    pub template_digest: String,
    pub accepted_by: String,
    pub view_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SparkDerivedViewsDdlAckEvidence {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub sink: String,
    pub ack_lsn: String,
    pub schema_version: String,
    pub template_digest: String,
    pub accepted_by: String,
    pub view_count: u32,
    pub release_gate: String,
}

pub fn spark_derived_views_ddl_ack_evidence(
    request: SparkDerivedViewsDdlAckRequest,
) -> Result<SparkDerivedViewsDdlAckEvidence, LakeError> {
    let evidence = SparkDerivedViewsDdlAckEvidence {
        source_id: request.source_id,
        database_id: request.database_id,
        dataset_id: request.dataset_id,
        barrier_id: request.barrier_id,
        sink: SPARK_DERIVED_VIEWS_SINK.to_string(),
        ack_lsn: request.ack_lsn,
        schema_version: request.schema_version,
        template_digest: request.template_digest,
        accepted_by: request.accepted_by,
        view_count: request.view_count,
        release_gate: RELEASE_GATE.to_string(),
    };
    evidence.validate()?;
    Ok(evidence)
}

impl SparkDerivedViewsDdlAckEvidence {
    pub async fn record_barrier_ack<S: DdlBarrierStore + ?Sized>(
        self,
        store: &S,
    ) -> trellara_checkpoint::Result<()> {
        store.record_ddl_barrier_ack(self.into_barrier_ack()).await
    }

    pub fn into_barrier_ack(self) -> DdlBarrierAck {
        let detail = format!(
            "Spark-derived views accepted {} regenerated templates; template_digest={}; accepted_by={}; release_gate={}",
            self.view_count, self.template_digest, self.accepted_by, self.release_gate
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
        validate_sha256_digest("template_digest", &self.template_digest)?;
        validate_clean_field("accepted_by", &self.accepted_by)?;
        if self.view_count == 0 {
            return Err(LakeError::InvalidDdlAckField {
                field: "view_count",
                reason: "must cover at least one derived view".to_string(),
            });
        }
        Ok(())
    }
}
