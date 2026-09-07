use crate::{CheckpointError, Result};

pub(crate) fn require_non_empty(value: &str, message: impl Into<String>) -> Result<()> {
    if value.trim().is_empty() {
        return Err(CheckpointError::Store(message.into()));
    }
    Ok(())
}

pub(crate) fn require_no_surrounding_whitespace(
    value: &str,
    message: impl Into<String>,
) -> Result<()> {
    if value.trim() != value {
        return Err(CheckpointError::Store(message.into()));
    }
    Ok(())
}

pub(crate) fn require_positive(value: i64, message: impl Into<String>) -> Result<()> {
    if value <= 0 {
        return Err(CheckpointError::Store(message.into()));
    }
    Ok(())
}

pub(crate) fn require_non_negative(value: i64, message: impl Into<String>) -> Result<()> {
    if value < 0 {
        return Err(CheckpointError::Store(message.into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_non_empty_rejects_whitespace() {
        assert!(matches!(
            require_non_empty("  ", "field must not be empty"),
            Err(CheckpointError::Store(message)) if message == "field must not be empty"
        ));
    }

    #[test]
    fn require_no_surrounding_whitespace_rejects_spaced_values() {
        assert!(matches!(
            require_no_surrounding_whitespace(" value ", "field must not be spaced"),
            Err(CheckpointError::Store(message)) if message == "field must not be spaced"
        ));
    }

    #[test]
    fn require_positive_rejects_zero() {
        assert!(matches!(
            require_positive(0, "count must be positive"),
            Err(CheckpointError::Store(message)) if message == "count must be positive"
        ));
    }

    #[test]
    fn require_non_negative_rejects_negative_values() {
        assert!(matches!(
            require_non_negative(-1, "count must not be negative"),
            Err(CheckpointError::Store(message)) if message == "count must not be negative"
        ));
    }
}
