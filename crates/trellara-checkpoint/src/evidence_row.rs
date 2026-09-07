use crate::{
    validation::{
        validate_reseed_event, validate_snapshot_handoff_event, validate_validation_event,
    },
    ReseedEvent, Result, SnapshotHandoffEvent, ValidationEvent,
};

pub(crate) fn reseed_event_from_parts(
    source_id: String,
    dataset_id: String,
    watermark_lsn: String,
    table_count: i64,
    copied_rows: i64,
    completed_at: String,
) -> Result<ReseedEvent> {
    let event = ReseedEvent {
        source_id,
        dataset_id,
        watermark_lsn,
        table_count,
        copied_rows,
        completed_at,
    };
    validate_reseed_event(&event)?;
    Ok(event)
}

pub(crate) fn snapshot_handoff_event_from_parts(
    source_id: String,
    dataset_id: String,
    relation: String,
    watermark_lsn: String,
    copied_rows: i64,
    completed_at: String,
) -> Result<SnapshotHandoffEvent> {
    let event = SnapshotHandoffEvent {
        source_id,
        dataset_id,
        relation,
        watermark_lsn,
        copied_rows,
        completed_at,
    };
    validate_snapshot_handoff_event(&event)?;
    Ok(event)
}

pub(crate) struct ValidationEventParts {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) source_watermark_lsn: String,
    pub(crate) target_watermark_lsn: String,
    pub(crate) converged: bool,
    pub(crate) table_count: i64,
    pub(crate) drift_count: i64,
    pub(crate) drift_relations: Vec<String>,
    pub(crate) evidence_sha256: Option<String>,
    pub(crate) completed_at: String,
}

pub(crate) fn validation_event_from_parts(parts: ValidationEventParts) -> Result<ValidationEvent> {
    let event = ValidationEvent {
        source_id: parts.source_id,
        dataset_id: parts.dataset_id,
        source_watermark_lsn: parts.source_watermark_lsn,
        target_watermark_lsn: parts.target_watermark_lsn,
        converged: parts.converged,
        table_count: parts.table_count,
        drift_count: parts.drift_count,
        drift_relations: parts.drift_relations,
        evidence_sha256: parts.evidence_sha256,
        completed_at: parts.completed_at,
    };
    validate_validation_event(&event)?;
    Ok(event)
}
