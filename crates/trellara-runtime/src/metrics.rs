pub const RUNTIME_METRIC_LIVE: &str = "trellara_runtime_live";
pub const RUNTIME_METRIC_READY: &str = "trellara_runtime_ready";
pub const RUNTIME_METRIC_PENDING_WORK: &str = "trellara_runtime_pending_work";
pub const RUNTIME_METRIC_RESTARTS_TOTAL: &str = "trellara_runtime_restarts_total";
pub const RUNTIME_METRIC_FAILURES_TOTAL: &str = "trellara_runtime_failures_total";
pub const RUNTIME_METRIC_LAST_SUCCESS_UNIXTIME: &str = "trellara_runtime_last_success_unixtime";
pub const RUNTIME_METRIC_LAST_DURABLE_LSN_BYTES: &str = "trellara_runtime_last_durable_lsn_bytes";

pub const RUNTIME_METRIC_LABELS: [&str; 3] = ["service", "source_id", "dataset_id"];
