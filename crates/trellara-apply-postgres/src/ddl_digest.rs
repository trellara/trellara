use sha2::{Digest, Sha256};

use crate::SqlStatement;

pub(crate) fn target_ddl_statement_digest(sql: &str) -> String {
    format!("{:x}", Sha256::digest(sql.as_bytes()))
}

pub(crate) fn target_ddl_transaction_digest(
    barrier_id: &str,
    statements: &[SqlStatement],
    statement_sha256s: &[String],
) -> String {
    let canonical = statements
        .iter()
        .zip(statement_sha256s.iter())
        .map(|(statement, digest)| {
            format!(
                "{}\n{}\n{}\n{}",
                statement.total_order, statement.operation, statement.sql, digest
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");
    format!(
        "{:x}",
        Sha256::digest(format!("{barrier_id}\n{canonical}").as_bytes())
    )
}

pub(crate) fn sha256_is_valid(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}
