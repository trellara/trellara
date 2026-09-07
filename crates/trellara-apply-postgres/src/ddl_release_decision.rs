use serde::Serialize;
use trellara_checkpoint::{
    lsn_shape_is_valid, parse_lsn, DdlBarrierReleaseBlocker, DdlBarrierSummary,
};
use trellara_protocol::DDL_PROPAGATION_CDC_BOUNDARY;

use crate::{ApplyError, Result};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TargetDmlReleaseDecision {
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub barrier_id: String,
    pub barrier_lsn: String,
    pub cdc_transaction_boundary: String,
    pub release_dml: bool,
    pub release_gate: String,
    pub blockers: Vec<String>,
    pub blocker_codes: Vec<String>,
    pub blocker_details: Vec<DdlBarrierReleaseBlocker>,
    pub evidence: Vec<String>,
}

impl TargetDmlReleaseDecision {
    pub fn require_released(&self) -> Result<()> {
        validate_release_boundary_contract(self)?;
        if self.release_dml {
            return Ok(());
        }
        Err(ApplyError::DdlDmlReleaseBlocked {
            barrier_id: self.barrier_id.clone(),
            blockers: self.blockers.clone(),
        })
    }

    pub fn require_released_at_boundary(&self, source_commit_lsn: &str) -> Result<()> {
        self.require_released()?;
        validate_release_lsn("barrier_lsn", &self.barrier_lsn, &self.barrier_id)?;
        validate_release_lsn("source_commit_lsn", source_commit_lsn, &self.barrier_id)?;
        if parse_lsn(&self.barrier_lsn) == parse_lsn(source_commit_lsn) {
            return Ok(());
        }
        Err(ApplyError::DdlDmlReleaseBlocked {
            barrier_id: self.barrier_id.clone(),
            blockers: vec![format!(
                "post-DDL DML source commit LSN {source_commit_lsn} does not match DDL barrier_lsn {}",
                self.barrier_lsn
            )],
        })
    }
}

fn validate_release_boundary_contract(decision: &TargetDmlReleaseDecision) -> Result<()> {
    if propagation_boundary_token(&decision.cdc_transaction_boundary).as_deref()
        == Some(DDL_PROPAGATION_CDC_BOUNDARY)
    {
        return Ok(());
    }
    Err(ApplyError::DdlDmlReleaseBlocked {
        barrier_id: decision.barrier_id.clone(),
        blockers: vec![format!(
            "post-DDL DML release is missing canonical propagation boundary {DDL_PROPAGATION_CDC_BOUNDARY}"
        )],
    })
}

fn propagation_boundary_token(cdc_transaction_boundary: &str) -> Option<String> {
    cdc_transaction_boundary.split(';').find_map(|part| {
        part.trim()
            .strip_prefix("propagation_boundary=")
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string)
    })
}

fn validate_release_lsn(field: &'static str, lsn: &str, barrier_id: &str) -> Result<()> {
    if lsn_shape_is_valid(lsn) && parse_lsn(lsn) != 0 {
        return Ok(());
    }
    Err(ApplyError::DdlDmlReleaseBlocked {
        barrier_id: barrier_id.to_string(),
        blockers: vec![format!(
            "post-DDL DML release {field} {lsn} must be a non-zero PostgreSQL LSN"
        )],
    })
}

pub fn target_ddl_release_decision(summary: &DdlBarrierSummary) -> TargetDmlReleaseDecision {
    TargetDmlReleaseDecision {
        source_id: summary.source_id.clone(),
        database_id: summary.database_id.clone(),
        dataset_id: summary.dataset_id.clone(),
        barrier_id: summary.barrier_id.clone(),
        barrier_lsn: summary.barrier_lsn.clone(),
        cdc_transaction_boundary: summary.cdc_transaction_boundary.clone(),
        release_dml: summary.release_dml,
        release_gate: "post_ddl_dml_release".to_string(),
        blockers: summary.release_blockers.clone(),
        blocker_codes: summary.release_blocker_codes.clone(),
        blocker_details: summary.release_blocker_details.clone(),
        evidence: summary
            .release_gates
            .iter()
            .map(|gate| {
                format!(
                    "{} satisfied={} evidence={}",
                    gate.name, gate.satisfied, gate.evidence
                )
            })
            .collect(),
    }
}
