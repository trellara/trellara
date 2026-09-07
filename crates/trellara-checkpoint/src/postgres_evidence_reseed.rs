use crate::evidence::ReseedEvent;
use crate::evidence_row::reseed_event_from_parts;
use crate::postgres::PostgresCheckpointStore;
use crate::postgres_evidence_sql::{INSERT_RESEED_EVENT, LOAD_LATEST_RESEED_EVENT};
use crate::types::FlowKey;
use crate::validation::validate_reseed_event;
use crate::Result;

impl PostgresCheckpointStore {
    pub async fn record_reseed_event(&self, event: ReseedEvent) -> Result<()> {
        validate_reseed_event(&event)?;
        self.client
            .execute(
                INSERT_RESEED_EVENT,
                &[
                    &event.source_id,
                    &event.dataset_id,
                    &event.watermark_lsn,
                    &event.table_count,
                    &event.copied_rows,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn load_latest_reseed_event(&self, flow: &FlowKey) -> Result<Option<ReseedEvent>> {
        self.client
            .query_opt(
                LOAD_LATEST_RESEED_EVENT,
                &[&flow.source_id, &flow.dataset_id],
            )
            .await?
            .map(|row| {
                reseed_event_from_parts(
                    row.get(0),
                    row.get(1),
                    row.get(2),
                    row.get(3),
                    row.get(4),
                    row.get(5),
                )
            })
            .transpose()
    }
}
