use tokio_postgres::Client;
#[cfg(test)]
pub(crate) use trellara_protocol::RelationId;
mod assembler;
mod assembler_boundary;
mod assembler_buffer;
mod assembler_config;
mod assembler_ddl;
mod assembler_envelope;
mod assembler_event_apply;
mod assembler_event_state;
mod assembler_events;
mod assembler_order;
mod assembler_pending_ddl;
mod assembler_schema_versions;
mod assembler_spill;
mod assembler_spill_config;
mod assembler_stream;
mod assembler_stream_events;
mod assembler_transaction;
mod capture_control;
mod capture_publication;
mod capture_relations;
mod capture_slot;
mod error;
mod lsn;
mod pgoutput;
mod pgoutput_changes;
mod pgoutput_event;
mod pgoutput_relation;
mod pgoutput_relation_fingerprint;
mod pgoutput_relation_registry;
mod pgoutput_stream_capture;
mod pgoutput_stream_start;
mod pgoutput_stream_state;
mod pgoutput_truncate;
mod pgoutput_tuple;
mod postgres_connect;
mod reader;
mod replication;
mod replication_config;
mod replication_error;
mod replication_feedback;
mod replication_frame;
mod replication_protocol;
mod replication_query;
mod replication_slot_row;
mod source;
mod source_ack_boundary;
mod source_discovery;
mod source_inspection;
mod source_postgres_risk_inspection;
mod source_postgres_risk_sql;
mod source_replica_identity_contract;
mod source_safety_inspection;
mod source_safety_queries;
mod source_slot_inspection;
mod source_subscription_inspection;
mod source_table_inspection;
mod subscription_conflicts;
mod test_decoding;
mod test_decoding_capture;
mod test_decoding_relation;
mod types;

pub use assembler::TransactionAssembler;
pub use assembler_config::TransactionAssemblerConfig;
pub use pgoutput::PgOutputDecoder;
pub use pgoutput_event::{LogicalEvent, ObservedDdlEvent};
pub(crate) use pgoutput_relation::PgOutputColumn;
pub(crate) use pgoutput_relation::PgOutputRelation;
pub use pgoutput_stream_capture::PgOutputStreamCapture;
pub(crate) use postgres_connect::connect_control_client;
#[cfg(test)]
pub(crate) use replication_protocol::replication_start_lsn;
pub(crate) use source_discovery::{inspect_all_user_tables, inspect_logical_replication_slots};
pub(crate) use source_inspection::{fail_on_preflight_issues, inspect_slot_status, inspect_tables};
pub(crate) use source_postgres_risk_inspection::attach_postgres_risk_evidence_to_slots;
pub use source_safety_inspection::{
    inspect_database_source_safety, DatabaseSourceSafetyInspection,
};
pub use source_safety_queries::{source_safety_read_only_queries, SourceSafetyReadOnlyQuery};
pub(crate) use source_slot_inspection::fail_on_slot_plugin_mismatch;
pub(crate) use source_subscription_inspection::inspect_subscription_conflict_stats;
pub use subscription_conflicts::{LogicalReplicationConflictCounts, SubscriptionConflictStats};
pub(crate) use test_decoding::parse_test_decoding_event;
pub use test_decoding_capture::TestDecodingCapture;
pub(crate) use types::{decode_replica_identity, quote_ident};
pub use types::{
    CapturedRelation, PgCaptureConfig, PgOutputProtocolConfig, ReplicationSlotStatus, TableColumn,
    TablePreflight, TableSelector, TransactionIdWraparoundStatus, WalHeadroomProjection,
    XminHorizonStatus, DEFAULT_PGOUTPUT_PROTOCOL_VERSION, DEFAULT_PGOUTPUT_STREAMING,
    DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES, MAX_STREAM_SPILL_THRESHOLD_CHANGES,
};

pub(crate) use reader::PgOutputReader;

pub use error::CaptureError;
pub use source::PgChangeSource;
pub use source::{
    CaptureBootstrap, ChangeSource, ExportedCaptureBootstrap, ExportedLogicalSlot,
    LogicalSlotBootstrap,
};
pub type Result<T> = std::result::Result<T, CaptureError>;

pub struct PgCapture {
    pub(crate) config: PgCaptureConfig,
    pub(crate) client: Client,
}

#[cfg(test)]
mod tests;
