use trellara_protocol::DdlEvent;

#[derive(Clone, Debug)]
pub(crate) struct PendingTransactionDdlEvent {
    pub(crate) total_order: u32,
    pub(crate) stream_subtransaction_id: Option<String>,
    pub(crate) event: DdlEvent,
}
