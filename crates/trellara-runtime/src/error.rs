use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum RuntimeContractError {
    #[error("unsupported runtime contract version {actual}; expected {expected}")]
    UnsupportedVersion { expected: u16, actual: u16 },
    #[error("runtime contract field {field} is invalid: {reason}")]
    InvalidField {
        field: &'static str,
        reason: &'static str,
    },
    #[error("runtime phase {phase} cannot report readiness {readiness}")]
    InvalidState {
        phase: &'static str,
        readiness: &'static str,
    },
}

pub type Result<T> = std::result::Result<T, RuntimeContractError>;
