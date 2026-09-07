use std::str::FromStr;

use bytes::BytesMut;
use postgres_protocol::message::frontend;
use tokio_postgres::Config as PostgresConfig;

pub(crate) use crate::replication_config::replication_endpoint;
#[cfg(test)]
pub(crate) use crate::replication_config::replication_tcp_host;
#[cfg(test)]
pub(crate) use crate::replication_feedback::standby_status_update_payload;
#[cfg(test)]
pub(crate) use crate::replication_protocol::validate_copy_both_response;
pub(crate) use crate::replication_protocol::PrimaryKeepalive;
#[cfg(test)]
pub(crate) use crate::replication_protocol::{parse_replication_copy_data, ReplicationCopyData};
#[cfg(test)]
pub(crate) use crate::replication_query::replication_options_sql;
#[cfg(test)]
pub(crate) use crate::replication_query::start_logical_replication_query;
use crate::source_ack_boundary::DurableSourceAckBoundary;
use crate::{CaptureError, Result};

mod auth;
mod commands;
mod stream;
mod stream_io;

use stream_io::ReplicationStream;

#[derive(Debug)]
pub(crate) struct ReplicationSlotCreation {
    pub(crate) slot_name: String,
    pub(crate) consistent_lsn: String,
    pub(crate) snapshot_name: Option<String>,
    pub(crate) output_plugin: String,
}

#[derive(Debug)]
pub(crate) struct ReplicationBootstrapConnection {
    stream: ReplicationStream,
    read_buf: BytesMut,
    latest_keepalive: Option<PrimaryKeepalive>,
    durable_source_ack: DurableSourceAckBoundary,
}

impl ReplicationBootstrapConnection {
    pub(crate) async fn connect(connection_uri: &str) -> Result<Self> {
        let config = PostgresConfig::from_str(connection_uri)
            .map_err(|error| CaptureError::InvalidConfig(error.to_string()))?;
        let endpoint = replication_endpoint(&config)?;
        let mut stream = ReplicationStream::connect(endpoint).await?;
        let user = config.get_user().ok_or_else(|| {
            CaptureError::InvalidConfig(
                "replication bootstrap requires an explicit database user".to_string(),
            )
        })?;
        let database = config.get_dbname().ok_or_else(|| {
            CaptureError::InvalidConfig(
                "replication bootstrap requires an explicit database name".to_string(),
            )
        })?;

        let mut startup = BytesMut::new();
        frontend::startup_message(
            [
                ("client_encoding", "UTF8"),
                ("user", user),
                ("database", database),
                ("replication", "database"),
                ("application_name", "trellara-replication-bootstrap"),
            ],
            &mut startup,
        )?;
        stream.write_all(&startup).await?;

        let mut connection = Self {
            stream,
            read_buf: BytesMut::new(),
            latest_keepalive: None,
            durable_source_ack: DurableSourceAckBoundary::default(),
        };
        connection.authenticate(&config, user).await?;
        Ok(connection)
    }
}
