use bytes::BytesMut;
use postgres_protocol::message::backend::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UnixStream};

use crate::replication_config::ReplicationEndpoint;
use crate::replication_frame::RawBackendMessage;
use crate::{CaptureError, Result};

#[derive(Debug)]
pub(super) enum ReplicationStream {
    Tcp(TcpStream),
    Unix(UnixStream),
}

impl ReplicationStream {
    pub(super) async fn connect(endpoint: ReplicationEndpoint) -> Result<Self> {
        match endpoint {
            ReplicationEndpoint::Tcp { host, port } => {
                Ok(Self::Tcp(TcpStream::connect((host.as_str(), port)).await?))
            }
            ReplicationEndpoint::Unix { path } => Ok(Self::Unix(UnixStream::connect(path).await?)),
        }
    }

    pub(super) async fn write_all(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.write_all(bytes).await,
            Self::Unix(stream) => stream.write_all(bytes).await,
        }
    }

    async fn read_buf(&mut self, buf: &mut BytesMut) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read_buf(buf).await,
            Self::Unix(stream) => stream.read_buf(buf).await,
        }
    }
}

pub(super) async fn read_message(
    stream: &mut ReplicationStream,
    read_buf: &mut BytesMut,
) -> Result<Message> {
    loop {
        if let Some(message) = Message::parse(read_buf)? {
            return Ok(message);
        }

        let read = stream.read_buf(read_buf).await?;
        if read == 0 {
            return Err(CaptureError::ReplicationProtocol(
                "replication connection closed".to_string(),
            ));
        }
    }
}

pub(super) async fn read_raw_message(
    stream: &mut ReplicationStream,
    read_buf: &mut BytesMut,
) -> Result<RawBackendMessage> {
    loop {
        if let Some(message) = RawBackendMessage::parse(read_buf)? {
            return Ok(message);
        }

        let read = stream.read_buf(read_buf).await?;
        if read == 0 {
            return Err(CaptureError::ReplicationProtocol(
                "replication connection closed".to_string(),
            ));
        }
    }
}
