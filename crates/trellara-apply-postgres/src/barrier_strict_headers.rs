use trellara_protocol::TransactionEnvelope;
use trellara_stream::{StreamHeader, StreamMessage};

use crate::barrier_header_lookup::optional_header;
use crate::barrier_header_payload::validate_header_payload_field;
use crate::barrier_strict_ddl_headers::validate_strict_ddl_proof_headers;
use crate::ApplyWorkerResult;

pub(crate) fn validate_strict_header_context(
    message: &StreamMessage,
    envelope: &TransactionEnvelope,
) -> ApplyWorkerResult<()> {
    validate_optional_header_payload_field(
        &message.headers,
        "source_id",
        "trellara.source_id",
        &envelope.source_id,
    )?;
    validate_optional_header_payload_field(
        &message.headers,
        "dataset_id",
        "trellara.dataset_id",
        &envelope.dataset_id,
    )?;
    validate_optional_header_payload_field(
        &message.headers,
        "database_id",
        "trellara.database_id",
        &envelope.database_id,
    )?;
    validate_optional_header_payload_field(
        &message.headers,
        "transaction_id",
        "trellara.transaction_id",
        &envelope.transaction_id,
    )?;
    validate_optional_header_payload_field(
        &message.headers,
        "commit_lsn",
        "trellara.commit_lsn",
        &envelope.commit_lsn,
    )?;
    validate_optional_header_payload_count(
        &message.headers,
        "ddl_event_count",
        "trellara.ddl_event_count",
        envelope.ddl_events.len(),
    )?;
    validate_optional_header_payload_field(
        &message.headers,
        "partitioned_scale_decision",
        "trellara.partitioned_scale_decision",
        &envelope.partitioned_scale_decision().to_string(),
    )?;
    validate_strict_ddl_proof_headers(&message.headers, envelope)
}

fn validate_optional_header_payload_count(
    headers: &[StreamHeader],
    field: &'static str,
    header_key: &'static str,
    payload_count: usize,
) -> ApplyWorkerResult<()> {
    validate_optional_header_payload_field(headers, field, header_key, &payload_count.to_string())
}

fn validate_optional_header_payload_field(
    headers: &[StreamHeader],
    field: &'static str,
    header_key: &'static str,
    payload: &str,
) -> ApplyWorkerResult<()> {
    let Some(header) = optional_header(headers, header_key)? else {
        return Ok(());
    };
    validate_header_payload_field(field, &header, payload)
}
