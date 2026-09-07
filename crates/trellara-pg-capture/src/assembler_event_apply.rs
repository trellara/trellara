use trellara_protocol::{Operation, RelationId, ReplicaIdentity, RowImage};

use crate::assembler_ddl::PendingDdlEvent;
use crate::{Result, TransactionAssembler};

impl TransactionAssembler {
    pub(crate) fn apply_change_event(
        &mut self,
        transaction_id: Option<String>,
        relation: RelationId,
        operation: Operation,
        replica_identity: ReplicaIdentity,
        before: Option<RowImage>,
        after: Option<RowImage>,
    ) -> Result<()> {
        let stream_subtransaction_id = transaction_id.clone();
        let open = self.open_transaction_for_event(transaction_id.as_deref())?;
        open.push_change(
            stream_subtransaction_id,
            relation,
            operation,
            replica_identity,
            before,
            after,
        )
    }

    pub(crate) fn apply_truncate_event(
        &mut self,
        transaction_id: Option<String>,
        relations: Vec<RelationId>,
    ) -> Result<()> {
        let stream_subtransaction_id = transaction_id.clone();
        let open = self.open_transaction_for_event(transaction_id.as_deref())?;
        for relation in relations {
            open.push_change(
                stream_subtransaction_id.clone(),
                relation,
                Operation::Truncate,
                ReplicaIdentity::Full,
                None,
                None,
            )?;
        }
        Ok(())
    }

    pub(crate) fn apply_ddl_event(
        &mut self,
        transaction_id: Option<String>,
        pending: PendingDdlEvent,
    ) -> Result<()> {
        let stream_subtransaction_id = transaction_id.clone();
        let open = self.open_transaction_for_event(transaction_id.as_deref())?;
        open.push_ddl(stream_subtransaction_id, pending.into_protocol_event())
    }
}
