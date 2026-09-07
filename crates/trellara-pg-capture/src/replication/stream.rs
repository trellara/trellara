use bytes::BytesMut;
use postgres_protocol::message::backend::Message;
use postgres_protocol::message::frontend;
use tracing::debug;

use crate::lsn::format_lsn;
use crate::replication_error::postgres_error_message_raw;
use crate::replication_feedback::{postgres_epoch_now_micros, standby_status_update_payload};
use crate::replication_frame::RawBackendMessage;
use crate::replication_protocol::{parse_replication_copy_data, ReplicationCopyData, XLogData};
use crate::{CaptureError, Result};

use super::stream_io;
use super::ReplicationBootstrapConnection;

impl ReplicationBootstrapConnection {
    pub(crate) async fn read_xlog_data(&mut self) -> Result<Option<XLogData>> {
        loop {
            let message = self.read_raw_message().await?;
            match message.tag {
                b'd' => match parse_replication_copy_data(&message.body)? {
                    ReplicationCopyData::XLogData(data) => {
                        debug!(
                            wal_start = %data.wal_start,
                            wal_end = %data.wal_end,
                            send_timestamp_ms = data.send_timestamp_ms,
                            "received xlog data"
                        );
                        return Ok(Some(data));
                    }
                    ReplicationCopyData::PrimaryKeepalive(keepalive) => {
                        let reply_requested = keepalive.reply_requested;
                        debug!(
                            wal_end = %keepalive.wal_end,
                            send_timestamp_ms = keepalive.send_timestamp_ms,
                            reply_requested,
                            "received primary keepalive"
                        );
                        self.latest_keepalive = Some(keepalive);
                        if reply_requested {
                            self.send_standby_status_update_lsn(
                                self.durable_source_ack.last_acknowledged_lsn(),
                            )
                            .await?;
                        }
                    }
                },
                b'E' => {
                    return Err(CaptureError::ReplicationProtocol(format!(
                        "replication stream error: {}",
                        postgres_error_message_raw(&message.body)?
                    )));
                }
                tag => {
                    return Err(CaptureError::ReplicationProtocol(format!(
                        "unexpected backend tag {} in replication stream",
                        tag as char
                    )));
                }
            }
        }
    }

    pub(crate) async fn send_standby_status_update(&mut self, lsn: &str) -> Result<()> {
        let lsn = self.durable_source_ack.acknowledge(lsn)?;
        self.send_standby_status_update_lsn(lsn).await
    }

    pub(super) async fn send_standby_status_update_lsn(&mut self, lsn: u64) -> Result<()> {
        let payload = standby_status_update_payload(lsn, postgres_epoch_now_micros(), false);
        let mut message = BytesMut::new();
        frontend::CopyData::new(payload.freeze())?.write(&mut message);
        self.stream.write_all(&message).await?;
        debug!(lsn = %format_lsn(lsn), "sent standby status update");
        Ok(())
    }

    pub(super) async fn read_message(&mut self) -> Result<Message> {
        stream_io::read_message(&mut self.stream, &mut self.read_buf).await
    }

    pub(super) async fn read_raw_message(&mut self) -> Result<RawBackendMessage> {
        stream_io::read_raw_message(&mut self.stream, &mut self.read_buf).await
    }
}
