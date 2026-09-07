use std::future::pending;
use std::time::Duration;

use tokio::sync::mpsc;
use trellara_runtime::{RuntimePhase, RuntimeService};

use super::*;

struct IdleThenWork {
    calls: u8,
    shutdown: mpsc::UnboundedSender<ServiceSignal>,
}

struct CountingWorker {
    calls: u8,
}

impl ContinuousWorker for CountingWorker {
    type Step = &'static str;

    fn run_once(&mut self) -> WorkerFuture<'_, Self::Step> {
        self.calls += 1;
        Box::pin(async { Ok(None) })
    }
}

impl ContinuousWorker for IdleThenWork {
    type Step = &'static str;

    fn run_once(&mut self) -> WorkerFuture<'_, Self::Step> {
        self.calls += 1;
        match self.calls {
            1 => Box::pin(async { Ok(None) }),
            2 => Box::pin(async { Ok(Some("0/10")) }),
            _ => {
                let _ = self.shutdown.send(ServiceSignal::Shutdown);
                Box::pin(pending())
            }
        }
    }
}

#[tokio::test]
async fn session_keeps_polling_after_idle_and_drains_on_shutdown() {
    let (sender, mut signals) = mpsc::unbounded_channel();
    let mut worker = IdleThenWork {
        calls: 0,
        shutdown: sender,
    };
    let state = ServiceState::new(RuntimeService::Relay, "source-a", "orders");
    let options = ServiceRuntimeOptions {
        idle_poll: Duration::from_millis(1),
        retry_initial: Duration::from_millis(1),
        retry_max: Duration::from_millis(2),
        shutdown_grace: Duration::from_millis(1),
    };
    let mut recorded = Vec::new();
    let (exit, made_progress) =
        run_service_session(&mut worker, &state, &mut signals, options, &mut |step| {
            recorded.push(step);
            Ok(Some(step.to_string()))
        })
        .await;

    assert_eq!(exit, SessionExit::Shutdown);
    assert!(made_progress);
    assert_eq!(recorded, vec!["0/10"]);
    assert_eq!(state.snapshot().health.phase, RuntimePhase::Draining);
    assert!(!state.snapshot().health.accepting_work);
}

#[tokio::test]
async fn queued_shutdown_stops_admission_before_polling_worker() {
    let (sender, mut signals) = mpsc::unbounded_channel();
    sender
        .send(ServiceSignal::Shutdown)
        .expect("queue shutdown");
    let mut worker = CountingWorker { calls: 0 };
    let state = ServiceState::new(RuntimeService::Applier, "source-a", "orders");
    let options = ServiceRuntimeOptions {
        idle_poll: Duration::from_millis(1),
        retry_initial: Duration::from_millis(1),
        retry_max: Duration::from_millis(2),
        shutdown_grace: Duration::from_millis(1),
    };

    let (exit, made_progress) =
        run_service_session(&mut worker, &state, &mut signals, options, &mut |_| {
            Ok(None)
        })
        .await;

    assert_eq!(exit, SessionExit::Shutdown);
    assert!(!made_progress);
    assert_eq!(worker.calls, 0);
    assert_eq!(state.snapshot().health.phase, RuntimePhase::Draining);
}
