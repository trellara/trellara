use std::path::PathBuf;

use tokio_postgres::config::{Host, SslMode};
use tokio_postgres::Config as PostgresConfig;

use crate::{CaptureError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplicationEndpoint {
    Tcp { host: String, port: u16 },
    Unix { path: PathBuf },
}

pub(crate) fn replication_endpoint(config: &PostgresConfig) -> Result<ReplicationEndpoint> {
    if config.get_ssl_mode() == SslMode::Require {
        return Err(CaptureError::InvalidConfig(
            "replication bootstrap does not support sslmode=require yet".to_string(),
        ));
    }

    let port = config.get_ports().first().copied().unwrap_or(5432);
    if !config.get_hostaddrs().is_empty() {
        return Ok(ReplicationEndpoint::Tcp {
            host: config.get_hostaddrs()[0].to_string(),
            port,
        });
    }

    match config.get_hosts().first() {
        Some(Host::Tcp(host)) => Ok(ReplicationEndpoint::Tcp {
            host: host.clone(),
            port,
        }),
        Some(Host::Unix(path)) => Ok(ReplicationEndpoint::Unix {
            path: unix_socket_file(path.clone(), port),
        }),
        None => Ok(ReplicationEndpoint::Tcp {
            host: "localhost".to_string(),
            port,
        }),
    }
}

#[cfg(test)]
pub(crate) fn replication_tcp_host(config: &PostgresConfig) -> Result<String> {
    match replication_endpoint(config)? {
        ReplicationEndpoint::Tcp { host, .. } => Ok(host),
        ReplicationEndpoint::Unix { .. } => Err(CaptureError::InvalidConfig(
            "replication bootstrap endpoint is a Unix socket, not a TCP host".to_string(),
        )),
    }
}

fn unix_socket_file(mut directory: PathBuf, port: u16) -> PathBuf {
    directory.push(format!(".s.PGSQL.{port}"));
    directory
}
