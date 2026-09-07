use crate::ddl::{ddl_sql_safety::validate_safe_executable_ddl_sql, RELEASE_GATE};
use crate::ddl_plan_digest_validation::validate_plan_digests;
use crate::{
    ApplyError, Result, TargetDdlApplyPlan, TargetDdlTransactionPlan,
    TARGET_DDL_TRANSACTION_BOUNDARY,
};

pub(crate) fn validate_ddl_apply_plan_header(plan: &TargetDdlApplyPlan) -> Result<()> {
    if plan.barrier_id.trim().is_empty() {
        return Err(ApplyError::MissingDdlField {
            field: "barrier_id",
        });
    }
    if plan.barrier_id != plan.barrier_id.trim() {
        return Err(ApplyError::InvalidDdlField {
            field: "barrier_id",
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    if plan.transaction_boundary_rule != TARGET_DDL_TRANSACTION_BOUNDARY {
        return Err(ApplyError::MissingDdlField {
            field: "transaction_boundary_rule",
        });
    }
    if !plan.blockers.is_empty() {
        return Err(ApplyError::DdlApplyBlocked {
            barrier_id: plan.barrier_id.clone(),
            blockers: plan.blockers.clone(),
        });
    }
    if plan.release_gate != RELEASE_GATE {
        return Err(ApplyError::MissingDdlField {
            field: "post_ddl_dml_release",
        });
    }
    if plan.statements.is_empty() {
        return Err(ApplyError::MissingDdlField {
            field: "statements",
        });
    }
    Ok(())
}

pub(crate) fn validate_ddl_transaction_plan(plan: &TargetDdlTransactionPlan) -> Result<()> {
    if plan.barrier_id.trim().is_empty() {
        return Err(ApplyError::MissingDdlField {
            field: "barrier_id",
        });
    }
    if plan.barrier_id != plan.barrier_id.trim() {
        return Err(ApplyError::InvalidDdlField {
            field: "barrier_id",
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    if plan.release_gate != RELEASE_GATE {
        return Err(ApplyError::MissingDdlField {
            field: "post_ddl_dml_release",
        });
    }
    if plan.statements.is_empty() {
        return Err(ApplyError::MissingDdlField {
            field: "statements",
        });
    }
    if plan.statement_count != plan.statements.len() {
        return Err(ApplyError::InvalidDdlField {
            field: "statement_count",
            reason: format!(
                "expected {}, got {}",
                plan.statements.len(),
                plan.statement_count
            ),
        });
    }
    for (index, statement) in plan.statements.iter().enumerate() {
        let expected_total_order = (index + 1) as u32;
        if statement.total_order != expected_total_order {
            return Err(ApplyError::InvalidDdlField {
                field: "statement.total_order",
                reason: format!(
                    "expected {}, got {}",
                    expected_total_order, statement.total_order
                ),
            });
        }
        if statement.operation != "ddl" {
            return Err(ApplyError::InvalidDdlField {
                field: "statement.operation",
                reason: "must be ddl".to_string(),
            });
        }
        if !statement.values.is_empty() {
            return Err(ApplyError::InvalidDdlField {
                field: "statement.values",
                reason: "DDL statements must not carry bind values".to_string(),
            });
        }
        if statement.requires_row_match {
            return Err(ApplyError::InvalidDdlField {
                field: "statement.requires_row_match",
                reason: "DDL statements must not require row matches".to_string(),
            });
        }
        validate_safe_executable_ddl_sql(statement)?;
    }
    validate_plan_digests(plan)?;
    Ok(())
}
