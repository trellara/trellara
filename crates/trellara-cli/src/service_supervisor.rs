use std::future::Future;

use crate::{
    run_service_session, ContinuousWorker, RetryBackoff, ServiceRuntime, ServiceSignal, SessionExit,
};

pub(crate) async fn supervise_service<S, A, Build, BuildFuture, Record>(
    mut runtime: ServiceRuntime,
    mut build: Build,
    mut accumulator: A,
    mut record: Record,
) -> A
where
    S: Send + 'static,
    Build: FnMut() -> BuildFuture,
    BuildFuture: Future<Output = std::result::Result<Box<dyn ContinuousWorker<Step = S>>, String>>,
    Record: FnMut(&mut A, S) -> std::result::Result<Option<String>, String>,
{
    let mut backoff = RetryBackoff::new(runtime.options.retry_initial, runtime.options.retry_max);
    let mut restarted = false;
    loop {
        runtime.state.mark_starting(restarted);
        let worker = match build_or_control(&mut runtime, build()).await {
            BuildExit::Worker(worker) => worker,
            BuildExit::Reload => {
                runtime.state.mark_draining(true);
                backoff.reset();
                restarted = true;
                continue;
            }
            BuildExit::Shutdown => {
                runtime.state.mark_draining(false);
                break;
            }
            BuildExit::Failed => {
                runtime.state.mark_failure("connect_error");
                restarted = true;
                let delay = backoff.take();
                report_retry("connect_error", delay);
                match wait_for_retry(&mut runtime, delay).await {
                    SessionExit::Shutdown => {
                        runtime.state.mark_draining(false);
                        break;
                    }
                    SessionExit::Reload => backoff.reset(),
                    SessionExit::Failed => {}
                }
                continue;
            }
        };
        let mut worker = worker;
        let mut record_step = |step| record(&mut accumulator, step);
        let (session_exit, made_progress) = run_service_session(
            worker.as_mut(),
            &runtime.state,
            &mut runtime.signals,
            runtime.options,
            &mut record_step,
        )
        .await;
        if made_progress {
            backoff.reset();
        }
        match session_exit {
            SessionExit::Shutdown => break,
            SessionExit::Reload => backoff.reset(),
            SessionExit::Failed => {
                let delay = backoff.take();
                report_retry("worker_error", delay);
                match wait_for_retry(&mut runtime, delay).await {
                    SessionExit::Shutdown => {
                        runtime.state.mark_draining(false);
                        break;
                    }
                    SessionExit::Reload => backoff.reset(),
                    SessionExit::Failed => {}
                }
            }
        }
        restarted = true;
    }
    runtime.stop().await;
    accumulator
}

enum BuildExit<S> {
    Worker(Box<dyn ContinuousWorker<Step = S>>),
    Reload,
    Shutdown,
    Failed,
}

async fn build_or_control<S, F>(runtime: &mut ServiceRuntime, build: F) -> BuildExit<S>
where
    S: Send + 'static,
    F: Future<Output = std::result::Result<Box<dyn ContinuousWorker<Step = S>>, String>>,
{
    tokio::pin!(build);
    tokio::select! {
        biased;
        signal = runtime.signals.recv() => match signal.unwrap_or(ServiceSignal::Shutdown) {
            ServiceSignal::Reload => BuildExit::Reload,
            ServiceSignal::Shutdown => BuildExit::Shutdown,
        },
        result = &mut build => match result {
            Ok(worker) => BuildExit::Worker(worker),
            Err(_) => BuildExit::Failed,
        }
    }
}

async fn wait_for_retry(runtime: &mut ServiceRuntime, delay: std::time::Duration) -> SessionExit {
    tokio::select! {
        _ = tokio::time::sleep(delay) => SessionExit::Failed,
        signal = runtime.signals.recv() => match signal.unwrap_or(ServiceSignal::Shutdown) {
            ServiceSignal::Reload => {
                runtime.state.mark_draining(true);
                SessionExit::Reload
            }
            ServiceSignal::Shutdown => SessionExit::Shutdown,
        }
    }
}

fn report_retry(reason_code: &'static str, delay: std::time::Duration) {
    eprintln!(
        "{}",
        serde_json::json!({
            "event": "runtime_retry",
            "reason_code": reason_code,
            "retry_after_ms": delay.as_millis(),
        })
    );
}
