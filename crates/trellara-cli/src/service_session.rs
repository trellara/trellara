use std::future::Future;
use std::pin::Pin;

use tokio::sync::mpsc;

use crate::{ServiceRuntimeOptions, ServiceSignal, ServiceState};

pub(crate) type WorkerFuture<'a, S> =
    Pin<Box<dyn Future<Output = std::result::Result<Option<S>, String>> + 'a>>;

pub(crate) trait ContinuousWorker: Send {
    type Step: Send;

    fn run_once(&mut self) -> WorkerFuture<'_, Self::Step>;

    fn pending_work(&self) -> u64 {
        0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SessionExit {
    Reload,
    Shutdown,
    Failed,
}

pub(crate) async fn run_service_session<S, Record>(
    worker: &mut dyn ContinuousWorker<Step = S>,
    state: &ServiceState,
    signals: &mut mpsc::UnboundedReceiver<ServiceSignal>,
    options: ServiceRuntimeOptions,
    record: &mut Record,
) -> (SessionExit, bool)
where
    S: Send,
    Record: FnMut(S) -> std::result::Result<Option<String>, String>,
{
    state.mark_ready();
    let mut made_progress = false;
    loop {
        match signals.try_recv() {
            Ok(signal) => return (begin_drain(state, signal), made_progress),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                return (begin_drain(state, ServiceSignal::Shutdown), made_progress);
            }
            Err(mpsc::error::TryRecvError::Empty) => {}
        }
        let mut operation = worker.run_once();
        let outcome = tokio::select! {
            biased;
            signal = signals.recv() => StepOutcome::Interrupted(signal.unwrap_or(ServiceSignal::Shutdown)),
            result = &mut operation => StepOutcome::Completed(result),
        };
        match outcome {
            StepOutcome::Completed(result) => {
                drop(operation);
                match complete_step(worker, state, record, result) {
                    Ok(true) => {
                        made_progress = true;
                        continue;
                    }
                    Ok(false) => {
                        if let Some(exit) = wait_while_idle(signals, options.idle_poll).await {
                            return (begin_drain(state, exit), made_progress);
                        }
                    }
                    Err(()) => return (SessionExit::Failed, made_progress),
                }
            }
            StepOutcome::Interrupted(signal) => {
                let exit = begin_drain(state, signal);
                let result = tokio::time::timeout(options.shutdown_grace, operation).await;
                if let Ok(result) = result {
                    if complete_step(worker, state, record, result) == Ok(true) {
                        made_progress = true;
                    }
                }
                return (newest_exit(exit, signals), made_progress);
            }
        }
    }
}

enum StepOutcome<S> {
    Completed(std::result::Result<Option<S>, String>),
    Interrupted(ServiceSignal),
}

fn complete_step<S, Record>(
    worker: &dyn ContinuousWorker<Step = S>,
    state: &ServiceState,
    record: &mut Record,
    result: std::result::Result<Option<S>, String>,
) -> std::result::Result<bool, ()>
where
    S: Send,
    Record: FnMut(S) -> std::result::Result<Option<String>, String>,
{
    match result {
        Ok(Some(step)) => match record(step) {
            Ok(lsn) => {
                state.mark_progress(worker.pending_work(), lsn.as_deref());
                Ok(true)
            }
            Err(_) => {
                state.mark_failure("runtime_stat_error");
                Err(())
            }
        },
        Ok(None) => {
            state.mark_progress(worker.pending_work(), None);
            Ok(false)
        }
        Err(_) => {
            state.mark_failure("worker_error");
            Err(())
        }
    }
}

async fn wait_while_idle(
    signals: &mut mpsc::UnboundedReceiver<ServiceSignal>,
    delay: std::time::Duration,
) -> Option<ServiceSignal> {
    tokio::select! {
        _ = tokio::time::sleep(delay) => None,
        signal = signals.recv() => Some(signal.unwrap_or(ServiceSignal::Shutdown)),
    }
}

fn begin_drain(state: &ServiceState, signal: ServiceSignal) -> SessionExit {
    match signal {
        ServiceSignal::Reload => {
            state.mark_draining(true);
            SessionExit::Reload
        }
        ServiceSignal::Shutdown => {
            state.mark_draining(false);
            SessionExit::Shutdown
        }
    }
}

fn newest_exit(
    current: SessionExit,
    signals: &mut mpsc::UnboundedReceiver<ServiceSignal>,
) -> SessionExit {
    let mut exit = current;
    while let Ok(signal) = signals.try_recv() {
        if signal == ServiceSignal::Shutdown {
            exit = SessionExit::Shutdown;
        }
    }
    exit
}

#[cfg(test)]
#[path = "tests/tests_service_session.rs"]
mod tests;
