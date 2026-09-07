use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn validate_required_clean_header_value(
    key: &'static str,
    value: &str,
) -> ApplyWorkerResult<()> {
    if value.trim().is_empty() {
        return Err(ApplyWorkerError::InvalidHeaderField {
            key,
            reason: "must not be empty".to_string(),
        });
    }
    if value != value.trim() {
        return Err(ApplyWorkerError::InvalidHeaderField {
            key,
            reason: "must not contain surrounding whitespace".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn validate_optional_clean_header_value(
    key: &'static str,
    value: &str,
) -> ApplyWorkerResult<()> {
    if value.is_empty() {
        Ok(())
    } else {
        validate_required_clean_header_value(key, value)
    }
}
