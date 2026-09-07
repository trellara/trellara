use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;
use trellara_runtime::RuntimeService;

use crate::{
    spawn_health_server, spawn_signal_listener, CliError, Result, ServiceRuntimeArgs,
    ServiceSignal, ServiceState,
};

#[derive(Copy, Clone, Debug)]
pub(crate) struct ServiceRuntimeOptions {
    pub(crate) idle_poll: Duration,
    pub(crate) retry_initial: Duration,
    pub(crate) retry_max: Duration,
    pub(crate) shutdown_grace: Duration,
}

impl ServiceRuntimeOptions {
    pub(crate) fn from_args(args: &ServiceRuntimeArgs) -> Result<Self> {
        if args.idle_poll_ms == 0 {
            return Err(invalid_duration("idle-poll-ms"));
        }
        if args.retry_initial_ms == 0 {
            return Err(invalid_duration("retry-initial-ms"));
        }
        if args.retry_max_ms < args.retry_initial_ms {
            return Err(CliError::InvalidConfig(
                "--retry-max-ms must be greater than or equal to --retry-initial-ms".to_string(),
            ));
        }
        if args.shutdown_grace_ms == 0 {
            return Err(invalid_duration("shutdown-grace-ms"));
        }
        Ok(Self {
            idle_poll: Duration::from_millis(args.idle_poll_ms),
            retry_initial: Duration::from_millis(args.retry_initial_ms),
            retry_max: Duration::from_millis(args.retry_max_ms),
            shutdown_grace: Duration::from_millis(args.shutdown_grace_ms),
        })
    }
}

pub(crate) struct ServiceRuntime {
    pub(crate) options: ServiceRuntimeOptions,
    pub(crate) state: ServiceState,
    pub(crate) signals: mpsc::UnboundedReceiver<ServiceSignal>,
    signal_task: JoinHandle<()>,
    health_shutdown: watch::Sender<bool>,
    health_task: JoinHandle<()>,
}

impl ServiceRuntime {
    pub(crate) async fn start(
        service: RuntimeService,
        source_id: &str,
        dataset_id: &str,
        listen: SocketAddr,
        options: ServiceRuntimeOptions,
    ) -> Result<Self> {
        let listener = TcpListener::bind(listen)
            .await
            .map_err(|source| CliError::RuntimeIo {
                operation: "health listener bind",
                source,
            })?;
        let state = ServiceState::new(service, source_id, dataset_id);
        let (signal_sender, signals) = mpsc::unbounded_channel();
        let signal_task = spawn_signal_listener(signal_sender);
        let (health_shutdown, health_receiver) = watch::channel(false);
        let health_task = spawn_health_server(listener, state.clone(), health_receiver);
        Ok(Self {
            options,
            state,
            signals,
            signal_task,
            health_shutdown,
            health_task,
        })
    }

    pub(crate) async fn stop(self) {
        self.state.mark_stopped();
        let _ = self.health_shutdown.send(true);
        self.signal_task.abort();
        let _ = self.health_task.await;
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct RetryBackoff {
    initial: Duration,
    maximum: Duration,
    next: Duration,
}

impl RetryBackoff {
    pub(crate) fn new(initial: Duration, maximum: Duration) -> Self {
        Self {
            initial,
            maximum,
            next: initial,
        }
    }

    pub(crate) fn take(&mut self) -> Duration {
        let delay = self.next;
        self.next = self.next.saturating_mul(2).min(self.maximum);
        delay
    }

    pub(crate) fn reset(&mut self) {
        self.next = self.initial;
    }
}

fn invalid_duration(flag: &str) -> CliError {
    CliError::InvalidConfig(format!("--{flag} must be greater than zero"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    use crate::{Cli, Command};

    #[test]
    fn retry_backoff_is_exponential_and_capped() {
        let mut backoff = RetryBackoff::new(Duration::from_millis(10), Duration::from_millis(25));
        assert_eq!(backoff.take(), Duration::from_millis(10));
        assert_eq!(backoff.take(), Duration::from_millis(20));
        assert_eq!(backoff.take(), Duration::from_millis(25));
        assert_eq!(backoff.take(), Duration::from_millis(25));
        backoff.reset();
        assert_eq!(backoff.take(), Duration::from_millis(10));
    }

    #[test]
    fn relay_and_apply_default_to_continuous_service_ports() {
        let relay =
            Cli::try_parse_from(["trellara", "relay", "--config", "flow.yml"]).expect("relay args");
        let apply =
            Cli::try_parse_from(["trellara", "apply", "--config", "flow.yml"]).expect("apply args");

        let Command::Relay(relay) = relay.command else {
            panic!("relay command");
        };
        assert_eq!(relay.max_transactions, 0);
        assert_eq!(relay.health_listen.port(), 9401);
        assert_eq!(relay.service.retry_initial_ms, 250);

        let Command::Apply(apply) = apply.command else {
            panic!("apply command");
        };
        assert_eq!(apply.max_messages, 0);
        assert_eq!(apply.health_listen.port(), 9402);
        assert_eq!(apply.service.retry_max_ms, 30_000);
    }
}
