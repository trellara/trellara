use thiserror::Error;

#[derive(Debug, Error)]
#[error("change {total_order} field {field} is invalid: {reason}")]
pub struct InvalidChangeFieldError {
    pub total_order: u32,
    pub field: &'static str,
    pub reason: String,
}
