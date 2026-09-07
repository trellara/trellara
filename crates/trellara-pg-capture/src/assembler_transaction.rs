use trellara_protocol::{
    DdlEvent, Operation, RelationId, RelationSchemaVersion, ReplicaIdentity, RowImage,
    TransactionEnvelope,
};

use crate::assembler_buffer::{PendingChange, PendingChangeBuffer};
use crate::assembler_config::TransactionAssemblerConfig;
use crate::assembler_envelope::transaction_into_envelope;
use crate::assembler_order::next_total_order;
use crate::assembler_pending_ddl::PendingTransactionDdlEvent;
use crate::Result;

#[derive(Debug)]
pub(crate) struct OpenTransaction {
    pub(crate) transaction_id: String,
    pub(crate) begin_lsn: String,
    pub(crate) changes: PendingChangeBuffer,
    pub(crate) ddl_events: Vec<PendingTransactionDdlEvent>,
}

impl OpenTransaction {
    pub(crate) fn push_change(
        &mut self,
        stream_subtransaction_id: Option<String>,
        relation: RelationId,
        operation: Operation,
        replica_identity: ReplicaIdentity,
        before: Option<RowImage>,
        after: Option<RowImage>,
    ) -> Result<()> {
        let total_order = self.next_event_order()?;
        self.changes.push(PendingChange {
            total_order,
            stream_subtransaction_id,
            relation,
            operation,
            replica_identity,
            before,
            after,
        })
    }

    pub(crate) fn push_ddl(
        &mut self,
        stream_subtransaction_id: Option<String>,
        mut event: DdlEvent,
    ) -> Result<()> {
        event.transaction_id = self.transaction_id.clone();
        let total_order = self.next_event_order()?;
        event.total_order = total_order;
        self.ddl_events.push(PendingTransactionDdlEvent {
            total_order,
            stream_subtransaction_id,
            event,
        });
        Ok(())
    }

    pub(crate) fn into_envelope(
        self,
        config: &TransactionAssemblerConfig,
        commit_lsn: String,
        commit_timestamp_ms: i64,
        schema_versions: Vec<RelationSchemaVersion>,
    ) -> Result<TransactionEnvelope> {
        transaction_into_envelope(
            self,
            config,
            commit_lsn,
            commit_timestamp_ms,
            schema_versions,
        )
    }

    pub(crate) fn changed_relation_oids(&self) -> Result<Vec<u32>> {
        let mut ordered_oids = self.changes.ordered_relation_oids()?;
        ordered_oids.extend(self.ddl_events.iter().filter_map(|event| {
            event
                .event
                .relation
                .as_ref()
                .map(|relation| (event.total_order, relation.oid))
        }));
        ordered_oids.sort_by_key(|(total_order, _)| *total_order);
        Ok(ordered_oids
            .into_iter()
            .map(|(_, relation_oid)| relation_oid)
            .collect())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.changes.is_empty() && self.ddl_events.is_empty()
    }

    pub(crate) fn retain_stream_subtransaction_changes(
        &mut self,
        subtransaction_id: &str,
    ) -> Result<()> {
        self.changes.retain(|change| {
            change.stream_subtransaction_id.as_deref() != Some(subtransaction_id)
        })?;
        self.ddl_events
            .retain(|event| event.stream_subtransaction_id.as_deref() != Some(subtransaction_id));
        Ok(())
    }

    fn next_event_order(&self) -> Result<u32> {
        next_total_order(
            &self.transaction_id,
            self.changes.len() + self.ddl_events.len(),
        )
    }
}
