use crate::{ApplyWorkerError, ApplyWorkerResult};

pub(crate) fn validate_header_payload_field(
    field: &'static str,
    header: &str,
    payload: &str,
) -> ApplyWorkerResult<()> {
    if header == payload {
        Ok(())
    } else {
        Err(ApplyWorkerError::HeaderPayloadMismatch {
            field,
            header: header.to_string(),
            payload: payload.to_string(),
        })
    }
}
