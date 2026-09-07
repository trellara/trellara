use crate::evidence::SnapshotHandoffEvent;
use crate::evidence_row::snapshot_handoff_event_from_parts;
use crate::postgres::PostgresCheckpointStore;
use crate::postgres_evidence_sql::{
    INSERT_SNAPSHOT_HANDOFF_EVENT, LOAD_LATEST_SNAPSHOT_HANDOFF_EVENT,
};
use crate::types::FlowKey;
use crate::validation::validate_snapshot_handoff_event;
use crate::Result;

impl PostgresCheckpointStore {
    pub async fn record_snapshot_handoff_event(&self, event: SnapshotHandoffEvent) -> Result<()> {
        validate_snapshot_handoff_event(&event)?;
        self.client
            .execute(
                INSERT_SNAPSHOT_HANDOFF_EVENT,
                &[
                    &event.source_id,
                    &event.dataset_id,
                    &event.relation,
                    &event.watermark_lsn,
                    &event.copied_rows,
                ],
            )
            .await?;
        Ok(())
    }

    pub async fn load_latest_snapshot_handoff_event(
        &self,
        flow: &FlowKey,
    ) -> Result<Option<SnapshotHandoffEvent>> {
        self.client
            .query_opt(
                LOAD_LATEST_SNAPSHOT_HANDOFF_EVENT,
                &[&flow.source_id, &flow.dataset_id],
            )
            .await?
            .map(|row| {
                snapshot_handoff_event_from_parts(
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
