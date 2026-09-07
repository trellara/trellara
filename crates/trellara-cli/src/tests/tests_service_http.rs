use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use trellara_runtime::RuntimeService;

use super::*;

#[test]
fn readiness_and_metrics_follow_shared_runtime_state() {
    let state = ServiceState::new(RuntimeService::Relay, "source-a", "orders");
    assert_eq!(response_for("/readyz", &state.snapshot()).0, 503);
    state.mark_ready();
    let snapshot = state.snapshot();
    assert_eq!(response_for("/readyz", &snapshot).0, 200);
    assert!(render_metrics(&snapshot).contains(
        "trellara_runtime_ready{service=\"relay\",source_id=\"source-a\",dataset_id=\"orders\"} 1"
    ));
}

#[tokio::test]
async fn health_server_exposes_live_ready_health_and_metrics_routes() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind health listener");
    let address = listener.local_addr().expect("health address");
    let state = ServiceState::new(RuntimeService::Applier, "source-a", "orders");
    let (shutdown, receiver) = watch::channel(false);
    let task = spawn_health_server(listener, state.clone(), receiver);

    assert!(request(address, "/livez").await.starts_with("HTTP/1.1 200"));
    assert!(request(address, "/readyz")
        .await
        .starts_with("HTTP/1.1 503"));
    state.mark_ready();
    assert!(request(address, "/readyz")
        .await
        .starts_with("HTTP/1.1 200"));
    let health = request(address, "/healthz").await;
    assert!(health.contains("\"phase\":\"running\""));
    let metrics = request(address, "/metrics").await;
    assert!(metrics.contains("trellara_runtime_live{service=\"applier\""));

    shutdown.send(true).expect("stop health server");
    task.await.expect("health server task");
}

async fn request(address: std::net::SocketAddr, path: &str) -> String {
    let mut stream = TcpStream::connect(address).await.expect("connect health");
    stream
        .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
        .await
        .expect("write request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .expect("read response");
    response
}
