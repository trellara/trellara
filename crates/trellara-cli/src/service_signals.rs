use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ServiceSignal {
    Reload,
    Shutdown,
}

pub(crate) fn spawn_signal_listener(
    sender: mpsc::UnboundedSender<ServiceSignal>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        #[cfg(unix)]
        listen_unix(sender).await;
        #[cfg(not(unix))]
        listen_ctrl_c(sender).await;
    })
}

#[cfg(unix)]
async fn listen_unix(sender: mpsc::UnboundedSender<ServiceSignal>) {
    use tokio::signal::unix::{signal, SignalKind};

    let Ok(mut terminate) = signal(SignalKind::terminate()) else {
        let _ = sender.send(ServiceSignal::Shutdown);
        return;
    };
    let Ok(mut hangup) = signal(SignalKind::hangup()) else {
        let _ = sender.send(ServiceSignal::Shutdown);
        return;
    };

    loop {
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                let _ = result;
                let _ = sender.send(ServiceSignal::Shutdown);
                return;
            }
            _ = terminate.recv() => {
                let _ = sender.send(ServiceSignal::Shutdown);
                return;
            }
            _ = hangup.recv() => {
                if sender.send(ServiceSignal::Reload).is_err() {
                    return;
                }
            }
        }
    }
}

#[cfg(not(unix))]
async fn listen_ctrl_c(sender: mpsc::UnboundedSender<ServiceSignal>) {
    let _ = tokio::signal::ctrl_c().await;
    let _ = sender.send(ServiceSignal::Shutdown);
}
