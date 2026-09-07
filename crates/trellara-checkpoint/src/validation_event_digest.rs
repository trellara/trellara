use crate::{CheckpointError, Result, ValidationEvent};

pub(crate) fn validate_evidence_digest(event: &ValidationEvent) -> Result<()> {
    let Some(digest) = event.evidence_sha256.as_deref() else {
        return Ok(());
    };
    if digest.trim() != digest {
        return Err(CheckpointError::Store(
            "validation event evidence_sha256 must not contain surrounding whitespace".to_string(),
        ));
    }
    if digest.len() != 64
        || !digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(CheckpointError::Store(
            "validation event evidence_sha256 must be a 64-character SHA-256 hex digest"
                .to_string(),
        ));
    }
    Ok(())
}
