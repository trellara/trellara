use bytes::BytesMut;
use fallible_iterator::FallibleIterator;
use postgres_protocol::authentication;
use postgres_protocol::authentication::sasl::{ChannelBinding, ScramSha256, SCRAM_SHA_256};
use postgres_protocol::message::backend::Message;
use postgres_protocol::message::frontend;
use tokio_postgres::Config as PostgresConfig;

use crate::replication_error::postgres_error_message;
use crate::{CaptureError, Result};

use super::ReplicationBootstrapConnection;

impl ReplicationBootstrapConnection {
    pub(super) async fn authenticate(&mut self, config: &PostgresConfig, user: &str) -> Result<()> {
        loop {
            match self.read_message().await? {
                Message::AuthenticationOk => {}
                Message::AuthenticationCleartextPassword => {
                    let password = config_password(config)?;
                    self.send_password(password).await?;
                }
                Message::AuthenticationMd5Password(body) => {
                    let password = authentication::md5_hash(
                        user.as_bytes(),
                        config_password(config)?,
                        body.salt(),
                    );
                    self.send_password(password.as_bytes()).await?;
                }
                Message::AuthenticationSasl(body) => {
                    self.authenticate_scram(config, &body).await?;
                }
                Message::ParameterStatus(_) | Message::BackendKeyData(_) => {}
                Message::ReadyForQuery(_) => return Ok(()),
                Message::ErrorResponse(body) => {
                    return Err(CaptureError::ReplicationProtocol(format!(
                        "replication startup failed: {}",
                        postgres_error_message(&body)?
                    )));
                }
                _ => {
                    return Err(CaptureError::ReplicationProtocol(
                        "unexpected message during replication startup".to_string(),
                    ));
                }
            }
        }
    }

    async fn authenticate_scram(
        &mut self,
        config: &PostgresConfig,
        body: &postgres_protocol::message::backend::AuthenticationSaslBody,
    ) -> Result<()> {
        let mut mechanisms = body.mechanisms();
        let mut supports_scram = false;
        while let Some(mechanism) = mechanisms.next()? {
            if mechanism == SCRAM_SHA_256 {
                supports_scram = true;
            }
        }
        if !supports_scram {
            return Err(CaptureError::ReplicationProtocol(
                "server did not offer SCRAM-SHA-256 authentication".to_string(),
            ));
        }

        let mut scram = ScramSha256::new(config_password(config)?, ChannelBinding::unsupported());
        let mut response = BytesMut::new();
        frontend::sasl_initial_response(SCRAM_SHA_256, scram.message(), &mut response)?;
        self.stream.write_all(&response).await?;

        match self.read_message().await? {
            Message::AuthenticationSaslContinue(body) => {
                scram.update(body.data())?;
            }
            Message::ErrorResponse(body) => {
                return Err(CaptureError::ReplicationProtocol(format!(
                    "replication authentication failed: {}",
                    postgres_error_message(&body)?
                )));
            }
            _ => {
                return Err(CaptureError::ReplicationProtocol(
                    "expected AuthenticationSaslContinue".to_string(),
                ))
            }
        }

        let mut response = BytesMut::new();
        frontend::sasl_response(scram.message(), &mut response)?;
        self.stream.write_all(&response).await?;

        match self.read_message().await? {
            Message::AuthenticationSaslFinal(body) => {
                scram.finish(body.data())?;
            }
            Message::ErrorResponse(body) => {
                return Err(CaptureError::ReplicationProtocol(format!(
                    "replication authentication failed: {}",
                    postgres_error_message(&body)?
                )));
            }
            _ => {
                return Err(CaptureError::ReplicationProtocol(
                    "expected AuthenticationSaslFinal".to_string(),
                ));
            }
        }
        Ok(())
    }

    async fn send_password(&mut self, password: &[u8]) -> Result<()> {
        let mut message = BytesMut::new();
        frontend::password_message(password, &mut message)?;
        self.stream.write_all(&message).await?;
        Ok(())
    }
}

fn config_password(config: &PostgresConfig) -> Result<&[u8]> {
    config.get_password().ok_or_else(|| {
        CaptureError::InvalidConfig(
            "replication bootstrap requires an explicit database password".to_string(),
        )
    })
}
