use super::TargetDdlStatement;
use crate::{ApplyError, Result, SqlStatement};

pub(crate) fn validate_safe_additive_sql(statement: &TargetDdlStatement) -> Result<()> {
    validate_safe_additive_sql_parts(&statement.change, &statement.sql)
}

pub(crate) fn validate_safe_executable_ddl_sql(statement: &SqlStatement) -> Result<()> {
    validate_safe_additive_sql_parts("executable target DDL statement", &statement.sql)
}

fn validate_safe_additive_sql_parts(change: &str, sql: &str) -> Result<()> {
    let sql = sql.trim();
    let normalized = sql.to_ascii_lowercase();
    let tokens = sql_tokens(&normalized);
    let without_final_semicolon = normalized.trim_end_matches(';');
    let semicolon_count = normalized.matches(';').count();
    let safe_add_column =
        without_final_semicolon.starts_with("alter table ") && normalized.contains(" add column ");

    if semicolon_count > 1 || (!normalized.ends_with(';') && semicolon_count == 1) {
        return unsafe_ddl_change(change, "contains more than one SQL statement");
    }
    if normalized.contains("--") || normalized.contains("/*") || normalized.contains("*/") {
        return unsafe_ddl_change(change, "contains SQL comments");
    }
    if !safe_add_column {
        return unsafe_ddl_change(
            change,
            "only additive ALTER TABLE ADD COLUMN is auto-applied",
        );
    }
    if has_token(&tokens, "drop")
        || has_token(&tokens, "rename")
        || has_token_pair(&tokens, "alter", "column")
        || has_token_pair(&tokens, "set", "not")
        || has_token_pair(&tokens, "not", "null")
        || has_token(&tokens, "default")
        || has_token(&tokens, "unique")
        || has_token(&tokens, "primary")
        || has_token(&tokens, "references")
        || has_token(&tokens, "check")
        || has_token(&tokens, "constraint")
        || has_token(&tokens, "generated")
        || has_token(&tokens, "identity")
    {
        return unsafe_ddl_change(change, "contains destructive or contract-changing DDL");
    }
    Ok(())
}

fn unsafe_ddl_change<T>(change: &str, reason: &str) -> Result<T> {
    Err(ApplyError::UnsafeDdlStatement {
        change: change.to_string(),
        reason: reason.to_string(),
    })
}

fn sql_tokens(normalized: &str) -> Vec<&str> {
    normalized
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .collect()
}

fn has_token(tokens: &[&str], expected: &str) -> bool {
    tokens.contains(&expected)
}

fn has_token_pair(tokens: &[&str], first: &str, second: &str) -> bool {
    tokens
        .windows(2)
        .any(|window| window[0] == first && window[1] == second)
}
