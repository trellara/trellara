use tokio_postgres::Transaction;
use trellara_protocol::POST_DDL_DML_RELEASE_GATE;

#[path = "ddl_sql_safety.rs"]
pub(crate) mod ddl_sql_safety;

use ddl_sql_safety::validate_safe_additive_sql;

use crate::ddl_digest::{target_ddl_statement_digest, target_ddl_transaction_digest};
use crate::executor::execute_statement;
use crate::{validate_ddl_apply_plan_header, validate_ddl_transaction_plan, Result, SqlStatement};

pub(crate) const RELEASE_GATE: &str = POST_DDL_DML_RELEASE_GATE;
pub(crate) const TARGET_DDL_TRANSACTION_BOUNDARY: &str =
    "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlApplyPlan {
    pub barrier_id: String,
    pub transaction_boundary_rule: String,
    pub blockers: Vec<String>,
    pub release_gate: String,
    pub statements: Vec<TargetDdlStatement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlStatement {
    pub change: String,
    pub sql: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlTransactionPlan {
    pub barrier_id: String,
    pub statement_count: usize,
    pub statements: Vec<SqlStatement>,
    pub release_gate: String,
    pub plan_sha256: String,
    pub statement_sha256s: Vec<String>,
    pub steps: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetDdlApplyOutcome {
    pub barrier_id: String,
    pub applied_statements: usize,
    pub release_gate: String,
    pub plan_sha256: String,
    pub statement_sha256s: Vec<String>,
}

pub fn plan_target_ddl_transaction(plan: TargetDdlApplyPlan) -> Result<TargetDdlTransactionPlan> {
    validate_ddl_apply_plan_header(&plan)?;
    let statements = plan
        .statements
        .iter()
        .enumerate()
        .map(|(index, statement)| ddl_sql_statement(index, statement))
        .collect::<Result<Vec<_>>>()?;
    let statement_sha256s = statements
        .iter()
        .map(|statement| target_ddl_statement_digest(&statement.sql))
        .collect::<Vec<_>>();
    let plan_sha256 =
        target_ddl_transaction_digest(&plan.barrier_id, &statements, &statement_sha256s);

    Ok(TargetDdlTransactionPlan {
        barrier_id: plan.barrier_id,
        statement_count: statements.len(),
        statements,
        release_gate: plan.release_gate,
        plan_sha256,
        statement_sha256s,
        steps: vec![
            "begin target schema transaction".to_string(),
            "execute validated additive DDL statements in order".to_string(),
            "commit only after target schema fingerprint preflight matches".to_string(),
            "ack target_postgres before post-DDL DML release".to_string(),
        ],
    })
}

pub async fn execute_target_ddl_transaction(
    transaction: &Transaction<'_>,
    plan: &TargetDdlTransactionPlan,
) -> Result<usize> {
    validate_ddl_transaction_plan(plan)?;
    for statement in &plan.statements {
        execute_statement(transaction, statement).await?;
    }
    Ok(plan.statements.len())
}

fn ddl_sql_statement(index: usize, statement: &TargetDdlStatement) -> Result<SqlStatement> {
    target_ddl_sql_statement((index + 1) as u32, statement)
}

pub(crate) fn target_ddl_sql_statement(
    total_order: u32,
    statement: &TargetDdlStatement,
) -> Result<SqlStatement> {
    validate_safe_additive_sql(statement)?;
    Ok(SqlStatement {
        sql: statement.sql.clone(),
        values: Vec::new(),
        requires_row_match: false,
        total_order,
        operation: "ddl",
    })
}
