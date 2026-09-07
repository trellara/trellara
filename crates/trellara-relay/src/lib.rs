mod checkpoint;
mod error;
mod messages;
mod messages_barrier;
mod mode;
mod native_feedback;
#[cfg(feature = "kafka")]
mod native_kafka;
#[cfg(feature = "kafka")]
mod native_publish_ledger;
#[cfg(feature = "kafka")]
mod native_publish_proof;
#[cfg(feature = "kafka")]
mod native_publish_proof_codec;
mod native_spool;
mod native_transport;
mod native_transport_error;
mod native_worker;
mod publish;
mod publish_proof;
mod relay;
mod source_ack_boundary;
mod source_ack_proof_validation;
mod stats;
mod stats_counter;

pub use error::{RelayError, Result};
pub use mode::RelayMode;
pub use native_feedback::{
    native_feedback_decision_from_relay_step, native_feedback_proof_from_relay_step,
};
#[cfg(feature = "kafka")]
pub use native_kafka::NativeKafkaRelayConfig;
#[cfg(feature = "kafka")]
pub use native_transport::run_native_kafka_relay;
pub use native_transport::{run_native_relay, NativeRelayServerConfig};
pub use native_transport_error::{NativeRelayTransportError, NativeRelayTransportResult};
pub use native_worker::{
    run_native_worker_once, run_native_worker_once_report, NativeWorkerRunOutcome,
    NativeWorkerRunReport, NativeWorkerRunStatus,
};
pub use publish_proof::{RelayPublishProof, RELAY_PUBLISH_PROOF_CONTRACT};
pub use relay::{load_source_checkpoint, Relay};
pub use source_ack_boundary::{
    PublishDestination, SourceAckBoundaryProof, SOURCE_ACK_BOUNDARY_CONTRACT,
};
pub use stats::{RelayRunStats, RelayStep};
pub(crate) use stats_counter::checked_relay_stat_add;

#[cfg(test)]
mod tests;
