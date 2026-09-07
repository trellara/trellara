mod config;
mod config_behavior;
mod ident;
mod slot_status;
mod source_metadata;

pub use config::{
    PgCaptureConfig, PgOutputProtocolConfig, TableSelector, DEFAULT_PGOUTPUT_PROTOCOL_VERSION,
    DEFAULT_PGOUTPUT_STREAMING, DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
    MAX_STREAM_SPILL_THRESHOLD_CHANGES,
};
pub(crate) use ident::{decode_replica_identity, quote_ident};
pub use slot_status::{
    ReplicationSlotStatus, TransactionIdWraparoundStatus, WalHeadroomProjection, XminHorizonStatus,
};
pub use source_metadata::{CapturedRelation, TableColumn, TablePreflight};
