use clap::Args;

#[derive(Clone, Debug, Args)]
pub struct DevStackArgs {
    #[arg(long, help = "Print the Docker Compose command without running it")]
    pub dry_run: bool,
    #[arg(long, help = "Also start the Kafka-compatible Redpanda service")]
    pub with_kafka: bool,
    #[arg(long, help = "Also start the relay and applier runtime profile")]
    pub runtime: bool,
}

#[derive(Clone, Debug, Args)]
pub struct DevDownArgs {
    #[arg(long, help = "Print the Docker Compose command without running it")]
    pub dry_run: bool,
    #[arg(long, help = "Remove named compose volumes")]
    pub volumes: bool,
}

#[derive(Clone, Debug, Args)]
pub struct DevLogsArgs {
    #[arg(long, help = "Print the Docker Compose command without running it")]
    pub dry_run: bool,
    #[arg(short, long, help = "Follow logs until interrupted")]
    pub follow: bool,
    #[arg(long, help = "Include Kafka-compatible Redpanda logs")]
    pub with_kafka: bool,
    #[arg(long, help = "Include relay and applier runtime logs")]
    pub runtime: bool,
}
