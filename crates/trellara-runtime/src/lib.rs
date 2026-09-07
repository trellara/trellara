//! Shared production runtime, observability, and release contracts.

mod error;
mod lifecycle;
mod metrics;
mod release;

pub use error::{Result, RuntimeContractError};
pub use lifecycle::{
    RuntimeHealth, RuntimePhase, RuntimeReadiness, RuntimeService, RUNTIME_CONTRACT_VERSION,
};
pub use metrics::{
    RUNTIME_METRIC_FAILURES_TOTAL, RUNTIME_METRIC_LABELS, RUNTIME_METRIC_LAST_DURABLE_LSN_BYTES,
    RUNTIME_METRIC_LAST_SUCCESS_UNIXTIME, RUNTIME_METRIC_LIVE, RUNTIME_METRIC_PENDING_WORK,
    RUNTIME_METRIC_READY, RUNTIME_METRIC_RESTARTS_TOTAL,
};
pub use release::{
    supports_external_postgres_major, supports_native_postgres_major, ReleaseArchitecture,
    ReleaseComponent, ReleaseOperatingSystem, ReleasePackageFormat, RELEASE_ARCHITECTURES,
    RELEASE_CONTRACT_VERSION, SUPPORTED_EXTERNAL_POSTGRES_MAJORS, SUPPORTED_NATIVE_POSTGRES_MAJORS,
};

#[cfg(test)]
mod tests;
