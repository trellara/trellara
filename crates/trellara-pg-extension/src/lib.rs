//! Native PostgreSQL extension boundary for Trellara.
//!
//! The default feature set builds only the PostgreSQL-independent contract so
//! the main workspace test workflow does not require PostgreSQL headers or
//! `pg_config`. Build an installable extension with exactly one PostgreSQL
//! major feature.

mod contract;
mod drain;
mod feedback;
mod handoff;
mod hooks;
mod logical_frame;
mod metadata;
#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
mod pg_background_worker;
#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
mod pg_guc;
#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
mod pg_output_plugin;
#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
mod pg_runtime;
#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
mod pg_shared_queue;
mod queue;
mod readiness;
mod relay_auth;
mod shared_memory;
#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
mod sql_api;
mod status;
mod wire;
mod worker;

pub const EXTENSION_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SUPPORTED_POSTGRES_MAJORS: [u16; 4] = [15, 16, 17, 18];
pub const HANDOFF_CONTRACT: &str = "bounded_shared_memory_queue";
pub const SOURCE_ACKNOWLEDGEMENT_CONTRACT: &str = "only_after_durable_stream_publish";

#[must_use]
fn runtime_data_plane_ready() -> bool {
    #[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
    {
        pg_runtime::data_plane_ready()
    }
    #[cfg(not(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18")))]
    {
        false
    }
}

pub use contract::{native_capture_contract, CaptureContractError, NativeCaptureContract};
pub use drain::{
    native_handoff_drain_batch, NativeHandoffDrainBatch, NativeHandoffDrainError,
    RELAY_HANDOFF_CONTRACT,
};
pub use feedback::{
    native_source_feedback_decision, NativePublishDestination, NativeSourceFeedbackDecision,
    NativeSourceFeedbackError, NativeSourceFeedbackProof,
};
pub use handoff::{
    native_committed_transaction_frame, NativeHandoffError, NativeHandoffFrame,
    NativeHandoffFrameKind, MAX_HANDOFF_PAYLOAD_BYTES,
};
pub use hooks::{
    native_hook_registration_plan, NativeHookRegistrationPlan, NativeHookRegistrationStep,
    HOOK_REGISTRATION_CONTRACT,
};
pub use logical_frame::{
    NativeLogicalColumnStatus, NativeLogicalFrameBuilder, NativeLogicalFrameError,
    NativeLogicalOperation, MAX_LOGICAL_FRAME_BYTES,
};
pub use metadata::{
    native_relation_metadata, NativeRelationMetadata, NativeRelationMetadataError,
    REQUIRED_REPLICA_IDENTITY,
};
pub use queue::{
    native_handoff_queue_admission, native_handoff_queue_config, NativeHandoffQueueAdmission,
    NativeHandoffQueueConfig, NativeHandoffQueueError, DEFAULT_HANDOFF_QUEUE_FRAMES,
    MAX_HANDOFF_QUEUE_FRAMES,
};
pub use readiness::{
    native_data_plane_readiness, NativeDataPlaneReadiness, NativeDataPlaneReadinessComponent,
};
pub use relay_auth::{
    native_relay_auth_plan, NativeRelayAuthError, NativeRelayAuthPlan, RELAY_AUTH_CONTRACT,
    RELAY_NETWORK_BOUNDARY,
};
pub use shared_memory::{
    native_shared_memory_lifecycle_gate, native_shared_memory_plan,
    NativeSharedMemoryLifecycleGate, NativeSharedMemoryPhase, NativeSharedMemoryPlan,
    NativeSharedMemoryPlanError, SHARED_MEMORY_LIFECYCLE_CONTRACT,
};
pub use status::{compiled_postgres_major, supports_postgres_major, NativeExtensionStatus};
pub use wire::{
    native_relay_ack, NativeRelayWireAck, NativeRelayWireError, NativeRelayWireFrame,
    NATIVE_RELAY_WIRE_VERSION,
};
pub use worker::{
    native_worker_supervision_decision, NativeWorkerSupervisionDecision,
    NativeWorkerSupervisionError, NativeWorkerSupervisionState, WORKER_SUPERVISION_CONTRACT,
};

#[cfg(any(
    all(
        feature = "pg15",
        any(feature = "pg16", feature = "pg17", feature = "pg18")
    ),
    all(feature = "pg16", any(feature = "pg17", feature = "pg18")),
    all(feature = "pg17", feature = "pg18")
))]
compile_error!("enable exactly one PostgreSQL feature: pg15, pg16, pg17, or pg18");

#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
::pgrx::pg_module_magic!(name, version);

#[cfg(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18"))]
#[pgrx::pg_guard]
pub extern "C-unwind" fn _PG_init() {
    pg_runtime::initialize();
}

#[cfg(all(
    feature = "pg15",
    not(any(feature = "pg16", feature = "pg17", feature = "pg18"))
))]
const COMPILED_POSTGRES_MAJOR: Option<u16> = Some(15);

#[cfg(all(
    feature = "pg16",
    not(any(feature = "pg15", feature = "pg17", feature = "pg18"))
))]
const COMPILED_POSTGRES_MAJOR: Option<u16> = Some(16);

#[cfg(all(
    feature = "pg17",
    not(any(feature = "pg15", feature = "pg16", feature = "pg18"))
))]
const COMPILED_POSTGRES_MAJOR: Option<u16> = Some(17);

#[cfg(all(
    feature = "pg18",
    not(any(feature = "pg15", feature = "pg16", feature = "pg17"))
))]
const COMPILED_POSTGRES_MAJOR: Option<u16> = Some(18);

#[cfg(not(any(feature = "pg15", feature = "pg16", feature = "pg17", feature = "pg18")))]
const COMPILED_POSTGRES_MAJOR: Option<u16> = None;

#[cfg(test)]
mod tests;
