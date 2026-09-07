use crate::ddl::RELEASE_GATE;
use crate::ddl_digest::sha256_is_valid;
use crate::{
    ddl_ack_validation::{
        canonical_ack_lsn, canonical_barrier_lsn, validate_ack_at_or_after_barrier,
        validate_no_surrounding_whitespace, validate_non_empty,
    },
    ApplyError, Result, TargetDdlAckContext, TargetDdlAckEvidence, TargetDdlApplyOutcome,
};

impl TargetDdlApplyOutcome {
    pub fn target_ack_evidence(
        self,
        source_id: impl Into<String>,
        dataset_id: impl Into<String>,
        ack_lsn: impl Into<String>,
        schema_version: impl Into<String>,
    ) -> Result<TargetDdlAckEvidence> {
        let dataset_id = dataset_id.into();
        self.target_ack_evidence_for_database(
            source_id,
            dataset_id.clone(),
            dataset_id,
            ack_lsn,
            schema_version,
        )
    }

    pub fn target_ack_evidence_for_database(
        self,
        source_id: impl Into<String>,
        database_id: impl Into<String>,
        dataset_id: impl Into<String>,
        ack_lsn: impl Into<String>,
        schema_version: impl Into<String>,
    ) -> Result<TargetDdlAckEvidence> {
        self.target_ack_evidence_from_context(TargetDdlAckContext::new(
            source_id,
            database_id,
            dataset_id,
            ack_lsn,
            schema_version,
        ))
    }

    pub fn target_ack_evidence_from_context(
        self,
        context: TargetDdlAckContext,
    ) -> Result<TargetDdlAckEvidence> {
        if self.release_gate != RELEASE_GATE {
            return Err(ApplyError::MissingDdlField {
                field: "post_ddl_dml_release",
            });
        }
        let TargetDdlAckContext {
            source_id,
            database_id,
            dataset_id,
            ack_lsn,
            barrier_lsn,
            schema_version,
        } = context;
        validate_non_empty("source_id", &source_id)?;
        validate_no_surrounding_whitespace("source_id", &source_id)?;
        validate_non_empty("database_id", &database_id)?;
        validate_no_surrounding_whitespace("database_id", &database_id)?;
        validate_non_empty("dataset_id", &dataset_id)?;
        validate_no_surrounding_whitespace("dataset_id", &dataset_id)?;
        validate_non_empty("barrier_id", &self.barrier_id)?;
        validate_no_surrounding_whitespace("barrier_id", &self.barrier_id)?;
        validate_non_empty("schema_version", &schema_version)?;
        validate_no_surrounding_whitespace("schema_version", &schema_version)?;
        let ack_lsn = canonical_ack_lsn(&ack_lsn)?;
        let barrier_lsn = if let Some(barrier_lsn) = barrier_lsn {
            let barrier_lsn = canonical_barrier_lsn(&barrier_lsn)?;
            validate_ack_at_or_after_barrier(&ack_lsn, &barrier_lsn)?;
            Some(barrier_lsn)
        } else {
            None
        };
        if self.applied_statements == 0 {
            return Err(ApplyError::InvalidDdlAckEvidence {
                field: "applied_statements",
                reason: "target_postgres ACK must prove at least one applied DDL statement"
                    .to_string(),
            });
        }
        validate_ack_digests(&self)?;
        Ok(TargetDdlAckEvidence {
            source_id,
            database_id,
            dataset_id,
            barrier_id: self.barrier_id,
            sink: "target_postgres".to_string(),
            ack_lsn,
            barrier_lsn,
            schema_version,
            applied_statements: self.applied_statements,
            release_gate: self.release_gate,
            plan_sha256: self.plan_sha256,
            statement_sha256s: self.statement_sha256s,
        })
    }
}

fn validate_ack_digests(outcome: &TargetDdlApplyOutcome) -> Result<()> {
    if !sha256_is_valid(&outcome.plan_sha256) {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field: "plan_sha256",
            reason: "must be a 64-character SHA-256 hex digest".to_string(),
        });
    }
    if outcome.statement_sha256s.len() != outcome.applied_statements {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field: "statement_sha256",
            reason: format!("expected {} statement digests", outcome.applied_statements),
        });
    }
    if !outcome
        .statement_sha256s
        .iter()
        .all(|digest| sha256_is_valid(digest))
    {
        return Err(ApplyError::InvalidDdlAckEvidence {
            field: "statement_sha256",
            reason: "every statement digest must be a 64-character SHA-256 hex digest".to_string(),
        });
    }
    Ok(())
}
