use crate::ddl_digest::{
    sha256_is_valid, target_ddl_statement_digest, target_ddl_transaction_digest,
};
use crate::{ApplyError, Result, TargetDdlTransactionPlan};

pub(crate) fn validate_plan_digests(plan: &TargetDdlTransactionPlan) -> Result<()> {
    if !sha256_is_valid(&plan.plan_sha256) {
        return Err(ApplyError::InvalidDdlField {
            field: "plan_sha256",
            reason: "must be a 64-character SHA-256 hex digest".to_string(),
        });
    }
    if plan.statement_sha256s.len() != plan.statement_count {
        return Err(ApplyError::InvalidDdlField {
            field: "statement_sha256",
            reason: format!("expected {} statement digests", plan.statement_count),
        });
    }
    if !plan
        .statement_sha256s
        .iter()
        .all(|digest| sha256_is_valid(digest))
    {
        return Err(ApplyError::InvalidDdlField {
            field: "statement_sha256",
            reason: "every statement digest must be a 64-character SHA-256 hex digest".to_string(),
        });
    }
    for (statement, digest) in plan.statements.iter().zip(plan.statement_sha256s.iter()) {
        if target_ddl_statement_digest(&statement.sql) != *digest {
            return Err(ApplyError::InvalidDdlField {
                field: "statement_sha256",
                reason: "must match the validated target DDL statement SQL".to_string(),
            });
        }
    }
    if plan.plan_sha256
        != target_ddl_transaction_digest(
            &plan.barrier_id,
            &plan.statements,
            &plan.statement_sha256s,
        )
    {
        return Err(ApplyError::InvalidDdlField {
            field: "plan_sha256",
            reason: "must match the validated target DDL transaction plan".to_string(),
        });
    }
    Ok(())
}
