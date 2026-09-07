use crate::{CliError, Result};

pub(crate) fn validate_sha256_digest(flag: &str, digest: &str) -> Result<()> {
    if digest.trim().is_empty() {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --{flag} must not be empty"
        )));
    }
    if digest.trim() != digest {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --{flag} must not contain surrounding whitespace"
        )));
    }
    if digest.len() != 64
        || !digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --{flag} must be a 64-character SHA-256 hex digest"
        )));
    }
    Ok(())
}

pub(crate) fn validate_statement_digests(digests: &[String]) -> Result<()> {
    if digests.is_empty() {
        return Err(CliError::InvalidConfig(
            "schema ddl-barrier ack --plan-sha256 requires at least one --statement-sha256"
                .to_string(),
        ));
    }
    for digest in digests {
        validate_sha256_digest("statement-sha256", digest)?;
    }
    Ok(())
}
