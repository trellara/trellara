use thiserror::Error;
use trellara_stream::StreamError;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("failed to read config {path}: {source}")]
    ReadConfig {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to read input {path}: {source}")]
    ReadInput {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write output {path}: {source}")]
    WriteOutput {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse config {path}: {source}")]
    ParseConfig {
        path: String,
        source: serde_yaml::Error,
    },
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("capture error: {0}")]
    Capture(#[from] trellara_pg_capture::CaptureError),
    #[error("checkpoint error: {0}")]
    Checkpoint(#[from] trellara_checkpoint::CheckpointError),
    #[error("relay error: {0}")]
    Relay(#[from] trellara_relay::RelayError),
    #[error("stream error: {0}")]
    Stream(#[from] StreamError),
    #[error("protocol error: {0}")]
    Protocol(#[from] trellara_protocol::ProtocolError),
    #[error("apply error: {0}")]
    Apply(#[from] trellara_apply_postgres::ApplyError),
    #[error("apply worker error: {0}")]
    ApplyWorker(#[from] trellara_apply_postgres::ApplyWorkerError),
    #[cfg(feature = "kafka")]
    #[error("kafka stream error: {0}")]
    Kafka(#[from] trellara_stream_kafka::KafkaStreamError),
    #[cfg(feature = "local-stream")]
    #[error("local stream error: {0}")]
    LocalStream(#[from] trellara_stream_local::LocalStreamError),
    #[error("verify error: {0}")]
    Verify(#[from] trellara_verify::VerifyError),
    #[error("lake planner error: {0}")]
    Lake(#[from] trellara_lake::LakeError),
    #[error("failed to run {command}: {source}")]
    RunCommand {
        command: String,
        source: std::io::Error,
    },
    #[error("{command} exited with status {status}")]
    CommandFailed { command: String, status: String },
    #[error("count {field} cannot be negative: {value}")]
    NegativeCount { field: &'static str, value: i64 },
    #[error("count {field} value {value} exceeds i64 storage range")]
    CountOverflow { field: &'static str, value: u64 },
    #[error("runtime stat {field} overflowed u64")]
    RuntimeStatOverflow { field: &'static str },
    #[error("runtime {operation} failed: {source}")]
    RuntimeIo {
        operation: &'static str,
        source: std::io::Error,
    },
    #[error("failed to render output: {0}")]
    Render(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CliError>;

#[cfg(not(feature = "kafka"))]
pub(crate) fn kafka_feature_disabled() -> CliError {
    CliError::InvalidConfig(
        "stream.kind: kafka requires a trellara binary built with --features kafka".to_string(),
    )
}

#[cfg(not(feature = "local-stream"))]
pub(crate) fn local_stream_feature_disabled() -> CliError {
    CliError::InvalidConfig(
        "stream.kind: local requires a trellara binary built with --features local-stream"
            .to_string(),
    )
}
