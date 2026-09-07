use std::path::PathBuf;

use clap::{Args, Parser};

use crate::{QuickstartOutputFormat, TransactionInspectOutputFormat};

#[derive(Clone, Debug, Parser)]
pub struct BootstrapArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, default_value_t = true)]
    pub create_if_missing: bool,
}

#[derive(Clone, Debug, Parser)]
pub struct ApplyArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    /// Stop after this many messages; zero runs continuously until shutdown.
    #[arg(long, default_value_t = 0)]
    pub max_messages: u64,
    #[command(flatten)]
    pub service: ServiceRuntimeArgs,
    /// Address for liveness, readiness, health, and Prometheus metric endpoints.
    #[arg(long, default_value = "0.0.0.0:9402")]
    pub health_listen: std::net::SocketAddr,
}

#[derive(Clone, Debug, Parser)]
pub struct RelayArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long, default_value_t = true)]
    pub create_if_missing: bool,
    /// Stop after this many transactions; zero runs continuously until shutdown.
    #[arg(long, default_value_t = 0)]
    pub max_transactions: u64,
    #[command(flatten)]
    pub service: ServiceRuntimeArgs,
    /// Address for liveness, readiness, health, and Prometheus metric endpoints.
    #[arg(long, default_value = "0.0.0.0:9401")]
    pub health_listen: std::net::SocketAddr,
}

#[derive(Clone, Debug, Args)]
pub struct ServiceRuntimeArgs {
    /// Delay between polls when the source or stream is idle.
    #[arg(long, default_value_t = 250)]
    pub idle_poll_ms: u64,
    /// Initial reconnect delay after a failed worker attempt.
    #[arg(long, default_value_t = 250)]
    pub retry_initial_ms: u64,
    /// Maximum reconnect delay after consecutive failures.
    #[arg(long, default_value_t = 30_000)]
    pub retry_max_ms: u64,
    /// Time allowed for an in-flight durable boundary to finish during drain.
    #[arg(long, default_value_t = 30_000)]
    pub shutdown_grace_ms: u64,
}

impl Default for ServiceRuntimeArgs {
    fn default() -> Self {
        Self {
            idle_poll_ms: 250,
            retry_initial_ms: 250,
            retry_max_ms: 30_000,
            shutdown_grace_ms: 30_000,
        }
    }
}

#[derive(Clone, Debug, Parser)]
pub struct RunArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub local: bool,
    #[arg(long)]
    pub verify: bool,
    #[arg(long, value_enum, default_value_t = QuickstartOutputFormat::Json)]
    pub format: QuickstartOutputFormat,
    #[arg(long)]
    pub skip_snapshot: bool,
    #[arg(long, default_value = "local-run-snapshot")]
    pub snapshot_run_id: String,
    #[arg(long, default_value_t = true)]
    pub create_if_missing: bool,
    #[arg(long, default_value_t = 100)]
    pub max_transactions: u64,
    #[arg(long, default_value_t = 100)]
    pub max_messages: u64,
}

#[derive(Clone, Debug, Parser)]
pub struct ReseedArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub table: Option<String>,
}

#[derive(Clone, Debug, Parser)]
pub struct SnapshotArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long)]
    pub table: Option<String>,
    #[arg(long, default_value_t = true)]
    pub create_if_missing: bool,
    #[arg(long)]
    pub force: bool,
}

#[derive(Clone, Debug, Parser)]
pub struct VerifyArgs {
    #[arg(short, long)]
    pub config: PathBuf,
    #[arg(long)]
    pub table: Option<String>,
}

#[derive(Clone, Debug, Parser)]
pub struct InspectTransactionArgs {
    #[arg(short, long)]
    pub file: PathBuf,
    #[arg(long, value_enum, default_value_t = TransactionInspectOutputFormat::Json)]
    pub format: TransactionInspectOutputFormat,
}
