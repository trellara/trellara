use serde::{Deserialize, Serialize};

use crate::ddl::{RELEASE_GATE, TARGET_DDL_TRANSACTION_BOUNDARY};
use crate::{ApplyError, Result, TargetDdlApplyPlan, TargetDdlStatement};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TargetDdlApplyPlanEvidence {
    pub barrier_id: String,
    pub executable: bool,
    pub transaction_boundary_rule: String,
    pub blockers: Vec<String>,
    pub statements: Vec<TargetDdlStatementEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TargetDdlStatementEvidence {
    pub change: String,
    pub sink: String,
    pub sql: String,
    pub transaction_scope: String,
    pub release_gate_code: String,
}

pub fn target_ddl_apply_plan_from_evidence(
    evidence: TargetDdlApplyPlanEvidence,
) -> Result<TargetDdlApplyPlan> {
    validate_evidence_boundary(&evidence)?;
    if !evidence.executable {
        return Err(ApplyError::DdlApplyBlocked {
            barrier_id: evidence.barrier_id,
            blockers: evidence.blockers,
        });
    }

    let statements = evidence
        .statements
        .into_iter()
        .map(target_ddl_statement_from_evidence)
        .collect::<Result<Vec<_>>>()?;

    Ok(TargetDdlApplyPlan {
        barrier_id: evidence.barrier_id,
        transaction_boundary_rule: evidence.transaction_boundary_rule,
        blockers: evidence.blockers,
        release_gate: RELEASE_GATE.to_string(),
        statements,
    })
}

fn validate_evidence_boundary(evidence: &TargetDdlApplyPlanEvidence) -> Result<()> {
    if evidence.barrier_id.trim().is_empty() {
        return Err(ApplyError::MissingDdlField {
            field: "barrier_id",
        });
    }
    if evidence.barrier_id != evidence.barrier_id.trim() {
        return Err(ApplyError::InvalidDdlField {
            field: "barrier_id",
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    if evidence.transaction_boundary_rule != TARGET_DDL_TRANSACTION_BOUNDARY {
        return Err(ApplyError::MissingDdlField {
            field: "transaction_boundary_rule",
        });
    }
    if evidence.executable && evidence.statements.is_empty() {
        return Err(ApplyError::MissingDdlField {
            field: "statements",
        });
    }
    Ok(())
}

fn target_ddl_statement_from_evidence(
    evidence: TargetDdlStatementEvidence,
) -> Result<TargetDdlStatement> {
    validate_statement_evidence_boundary(&evidence)?;
    if evidence.sink != "target_postgres" {
        return unsafe_ddl_evidence(&evidence, "statement sink must be target_postgres");
    }
    if evidence.release_gate_code != RELEASE_GATE {
        return unsafe_ddl_evidence(&evidence, "statement must wait for post-DDL DML release");
    }
    if evidence.transaction_scope != "single target schema transaction before barrier ACK" {
        return unsafe_ddl_evidence(
            &evidence,
            "statement must execute in the single target schema transaction before barrier ACK",
        );
    }
    Ok(TargetDdlStatement {
        change: evidence.change,
        sql: evidence.sql,
    })
}

fn validate_statement_evidence_boundary(evidence: &TargetDdlStatementEvidence) -> Result<()> {
    validate_statement_evidence_field("change", &evidence.change)?;
    validate_statement_evidence_field("sink", &evidence.sink)?;
    validate_statement_evidence_field("sql", &evidence.sql)?;
    validate_statement_evidence_field("transaction_scope", &evidence.transaction_scope)?;
    validate_statement_evidence_field("release_gate_code", &evidence.release_gate_code)
}

fn validate_statement_evidence_field(field: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(ApplyError::MissingDdlField { field });
    }
    if value != value.trim() {
        return Err(ApplyError::InvalidDdlField {
            field,
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    Ok(())
}

fn unsafe_ddl_evidence<T>(statement: &TargetDdlStatementEvidence, reason: &str) -> Result<T> {
    Err(ApplyError::UnsafeDdlStatement {
        change: statement.change.clone(),
        reason: reason.to_string(),
    })
}
