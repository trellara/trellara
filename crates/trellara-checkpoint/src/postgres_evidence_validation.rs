use crate::evidence::ValidationEvent;
use crate::evidence_row::{validation_event_from_parts, ValidationEventParts};
use crate::postgres::PostgresCheckpointStore;
use crate::postgres_evidence_sql::{INSERT_VALIDATION_EVENT, LOAD_LATEST_VALIDATION_EVENT};
use crate::types::FlowKey;
use crate::validation::validate_validation_event;
use crate::Result;

impl PostgresCheckpointStore {
    pub async fn record_validation_event(&self, event: ValidationEvent) -> Result<()> {
        validate_validation_event(&event)?;
        self.client
            .execute(
                INSERT_VALIDATION_EVENT,
                &[
                    &event.source_id,
                    &event.dataset_id,
                    &event.source_watermark_lsn,
                    &event.target_watermark_lsn,
                    &event.converged,
                    &event.table_count,
                    &event.drift_count,
                    &event.drift_relations,
                    &event.evidence_sha256,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn load_latest_validation_event(
        &self,
        flow: &FlowKey,
    ) -> Result<Option<ValidationEvent>> {
        self.client
            .query_opt(
                LOAD_LATEST_VALIDATION_EVENT,
                &[&flow.source_id, &flow.dataset_id],
            )
            .await?
            .map(|row| {
                validation_event_from_parts(ValidationEventParts {
                    source_id: row.get(0),
                    dataset_id: row.get(1),
                    source_watermark_lsn: row.get(2),
                    target_watermark_lsn: row.get(3),
                    converged: row.get(4),
                    table_count: row.get(5),
                    drift_count: row.get(6),
                    drift_relations: row.get(7),
                    evidence_sha256: row.get(8),
                    completed_at: row.get(9),
                })
            })
            .transpose()
    }
}
