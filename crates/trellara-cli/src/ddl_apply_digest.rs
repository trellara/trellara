use sha2::{Digest, Sha256};

use crate::DdlApplyPlanStatement;

pub(crate) fn ddl_statement_digest(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}

pub(crate) fn ddl_apply_plan_digest(statements: &[DdlApplyPlanStatement]) -> String {
    let canonical = statements
        .iter()
        .map(|statement| {
            format!(
                "{}\n{}\n{}\n{}\n{}",
                statement.change,
                statement.sink,
                statement.sql,
                statement.release_gate_code,
                statement.statement_sha256
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");
    format!("{:x}", Sha256::digest(canonical.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statement(sql: &str) -> DdlApplyPlanStatement {
        DdlApplyPlanStatement {
            change: "add_nullable_column:public.sales.discount_code:text".to_string(),
            sink: "target_postgres".to_string(),
            sql: sql.to_string(),
            statement_sha256: ddl_statement_digest(sql),
            transaction_scope: "single target schema transaction before barrier ACK".to_string(),
            release_gate_code: "post_ddl_dml_release".to_string(),
            release_gate: "post_ddl_dml_release waits for ddl-barrier status".to_string(),
        }
    }

    #[test]
    fn ddl_apply_plan_digest_changes_when_statement_changes() {
        let first = ddl_apply_plan_digest(&[statement("select 1;")]);
        let second = ddl_apply_plan_digest(&[statement("select 2;")]);

        assert_ne!(first, second);
        assert_eq!(first.len(), 64);
    }
}
