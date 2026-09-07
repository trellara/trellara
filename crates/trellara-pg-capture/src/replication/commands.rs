use bytes::BytesMut;
use postgres_protocol::message::backend::Message;
use postgres_protocol::message::frontend;

use crate::replication_error::{postgres_error_message, postgres_error_message_raw};
use crate::replication_frame::RawBackendMessage;
use crate::replication_protocol::validate_copy_both_response;
use crate::replication_query::start_logical_replication_query;
use crate::replication_slot_row::replication_slot_creation_from_row;
use crate::{quote_ident, CaptureError, Result};

use super::{ReplicationBootstrapConnection, ReplicationSlotCreation};

impl ReplicationBootstrapConnection {
    pub(crate) async fn create_logical_slot_with_exported_snapshot(
        &mut self,
        slot_name: &str,
    ) -> Result<ReplicationSlotCreation> {
        self.send_query(&format!(
            "CREATE_REPLICATION_SLOT {} LOGICAL pgoutput (SNAPSHOT 'export')",
            quote_ident(slot_name)
        ))
        .await?;
        self.read_slot_creation_response().await
    }

    pub(crate) async fn start_logical_replication(
        &mut self,
        slot_name: &str,
        start_lsn: &str,
        options: &[(&str, &str)],
    ) -> Result<()> {
        let query = start_logical_replication_query(slot_name, start_lsn, options)?;
        self.send_query(&query).await?;
        self.read_start_replication_response().await
    }

    async fn send_query(&mut self, query: &str) -> Result<()> {
        let mut message = BytesMut::new();
        frontend::query(query, &mut message)?;
        self.stream.write_all(&message).await?;
        Ok(())
    }

    async fn read_slot_creation_response(&mut self) -> Result<ReplicationSlotCreation> {
        let mut row = None;
        loop {
            match self.read_message().await? {
                Message::RowDescription(_) => {}
                Message::DataRow(body) => {
                    row = Some(replication_slot_creation_from_row(&body)?);
                }
                Message::CommandComplete(_) => {}
                Message::ReadyForQuery(_) => {
                    return row.ok_or_else(|| {
                        CaptureError::ReplicationProtocol(
                            "CREATE_REPLICATION_SLOT returned no metadata row".to_string(),
                        )
                    });
                }
                Message::ErrorResponse(body) => {
                    return Err(CaptureError::ReplicationProtocol(format!(
                        "CREATE_REPLICATION_SLOT failed: {}",
                        postgres_error_message(&body)?
                    )));
                }
                _ => {
                    return Err(CaptureError::ReplicationProtocol(
                        "unexpected message during CREATE_REPLICATION_SLOT".to_string(),
                    ));
                }
            }
        }
    }

    async fn read_start_replication_response(&mut self) -> Result<()> {
        match self.read_raw_message().await? {
            RawBackendMessage { tag: b'W', body } => {
                validate_copy_both_response(&body)?;
                Ok(())
            }
            RawBackendMessage { tag: b'E', body } => {
                Err(CaptureError::ReplicationProtocol(format!(
                    "START_REPLICATION failed: {}",
                    postgres_error_message_raw(&body)?
                )))
            }
            RawBackendMessage { tag, .. } => Err(CaptureError::ReplicationProtocol(format!(
                "START_REPLICATION expected CopyBothResponse, got backend tag {}",
                tag as char
            ))),
        }
    }
}
