use std::io;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use trellara_runtime::{
    RuntimePhase, RUNTIME_METRIC_FAILURES_TOTAL, RUNTIME_METRIC_LAST_DURABLE_LSN_BYTES,
    RUNTIME_METRIC_LAST_SUCCESS_UNIXTIME, RUNTIME_METRIC_LIVE, RUNTIME_METRIC_PENDING_WORK,
    RUNTIME_METRIC_READY, RUNTIME_METRIC_RESTARTS_TOTAL,
};

use crate::{ServiceSnapshot, ServiceState};

pub(crate) fn spawn_health_server(
    listener: TcpListener,
    state: ServiceState,
    mut shutdown: watch::Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                result = listener.accept() => match result {
                    Ok((stream, _)) => {
                        let state = state.clone();
                        tokio::spawn(async move { let _ = serve_connection(stream, state).await; });
                    }
                    Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
                },
                result = shutdown.changed() => {
                    if result.is_err() || *shutdown.borrow() {
                        return;
                    }
                }
            }
        }
    })
}

async fn serve_connection(mut stream: TcpStream, state: ServiceState) -> io::Result<()> {
    let mut request = [0_u8; 2048];
    let read = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut request)).await;
    let count = match read {
        Ok(result) => result?,
        Err(_) => return Ok(()),
    };
    let path = request_path(&request[..count]);
    let snapshot = state.snapshot();
    let (status, content_type, body) = response_for(path, &snapshot);
    let reason = if status == 200 {
        "OK"
    } else if status == 404 {
        "Not Found"
    } else {
        "Service Unavailable"
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await
}

fn request_path(request: &[u8]) -> &str {
    std::str::from_utf8(request)
        .ok()
        .and_then(|request| request.lines().next())
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/")
}

fn response_for(path: &str, snapshot: &ServiceSnapshot) -> (u16, &'static str, String) {
    match path {
        "/livez" => probe_response(is_live(snapshot)),
        "/readyz" => probe_response(snapshot.health.accepting_work),
        "/healthz" => (
            200,
            "application/json",
            serde_json::to_string(snapshot).unwrap_or_else(|_| "{}".to_string()),
        ),
        "/metrics" => (200, "text/plain; version=0.0.4", render_metrics(snapshot)),
        _ => (404, "text/plain", "not found\n".to_string()),
    }
}

fn probe_response(healthy: bool) -> (u16, &'static str, String) {
    if healthy {
        (200, "text/plain", "ok\n".to_string())
    } else {
        (503, "text/plain", "not ready\n".to_string())
    }
}

fn is_live(snapshot: &ServiceSnapshot) -> bool {
    !matches!(
        snapshot.health.phase,
        RuntimePhase::Stopped | RuntimePhase::Failed
    )
}

fn render_metrics(snapshot: &ServiceSnapshot) -> String {
    let health = &snapshot.health;
    let labels = format!(
        "service=\"{}\",source_id=\"{}\",dataset_id=\"{}\"",
        service_label(health.service),
        escape_label(&health.source_id),
        escape_label(&health.dataset_id)
    );
    let live = u8::from(is_live(snapshot));
    let ready = u8::from(health.accepting_work);
    format!(
        "{RUNTIME_METRIC_LIVE}{{{labels}}} {live}\n\
         {RUNTIME_METRIC_READY}{{{labels}}} {ready}\n\
         {RUNTIME_METRIC_PENDING_WORK}{{{labels}}} {}\n\
         {RUNTIME_METRIC_RESTARTS_TOTAL}{{{labels}}} {}\n\
         {RUNTIME_METRIC_FAILURES_TOTAL}{{{labels}}} {}\n\
         {RUNTIME_METRIC_LAST_SUCCESS_UNIXTIME}{{{labels}}} {}\n\
         {RUNTIME_METRIC_LAST_DURABLE_LSN_BYTES}{{{labels}}} {}\n",
        health.pending_work,
        snapshot.restarts_total,
        snapshot.failures_total,
        snapshot.last_success_unixtime,
        snapshot.last_durable_lsn_bytes,
    )
}

fn service_label(service: trellara_runtime::RuntimeService) -> &'static str {
    match service {
        trellara_runtime::RuntimeService::Relay => "relay",
        trellara_runtime::RuntimeService::Applier => "applier",
        trellara_runtime::RuntimeService::IcebergWriter => "iceberg_writer",
    }
}

fn escape_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
#[path = "tests/tests_service_http.rs"]
mod tests;
