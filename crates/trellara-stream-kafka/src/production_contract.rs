use serde::{Deserialize, Serialize};

use crate::KafkaStreamError;

pub const KAFKA_PRODUCTION_CONTRACT_VERSION: u16 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum KafkaSecretRef {
    EnvironmentVariable { name: String },
    File { path: String },
}

impl KafkaSecretRef {
    fn validate(&self, field: &'static str) -> Result<(), KafkaStreamError> {
        match self {
            Self::EnvironmentVariable { name } => {
                let mut bytes = name.bytes();
                let valid_start = bytes
                    .next()
                    .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');
                let valid_tail = bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
                if valid_start && valid_tail {
                    Ok(())
                } else {
                    Err(invalid(
                        field,
                        "environment variable must use an ASCII shell identifier",
                    ))
                }
            }
            Self::File { path } if path.starts_with('/') && path.len() > 1 => Ok(()),
            Self::File { .. } => Err(invalid(field, "secret file path must be absolute")),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KafkaSaslMechanism {
    Plain,
    ScramSha256,
    ScramSha512,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum KafkaAuthenticationContract {
    MutualTls {
        client_certificate: KafkaSecretRef,
        client_key: KafkaSecretRef,
    },
    SaslTls {
        mechanism: KafkaSaslMechanism,
        username: KafkaSecretRef,
        password: KafkaSecretRef,
    },
}

impl KafkaAuthenticationContract {
    fn validate(&self) -> Result<(), KafkaStreamError> {
        match self {
            Self::MutualTls {
                client_certificate,
                client_key,
            } => {
                client_certificate.validate("authentication.client_certificate")?;
                client_key.validate("authentication.client_key")
            }
            Self::SaslTls {
                username, password, ..
            } => {
                username.validate("authentication.username")?;
                password.validate("authentication.password")
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KafkaProductionContract {
    pub contract_version: u16,
    pub ca_certificate: KafkaSecretRef,
    pub authentication: KafkaAuthenticationContract,
    pub replication_factor: u16,
    pub min_insync_replicas: u16,
    pub producer_acks_all: bool,
    pub producer_idempotence: bool,
    pub consumer_auto_commit: bool,
    pub store_offset_after_apply: bool,
    pub synchronous_offset_commit: bool,
}

impl KafkaProductionContract {
    #[must_use]
    pub fn sasl_tls(
        ca_certificate: KafkaSecretRef,
        mechanism: KafkaSaslMechanism,
        username: KafkaSecretRef,
        password: KafkaSecretRef,
    ) -> Self {
        Self {
            contract_version: KAFKA_PRODUCTION_CONTRACT_VERSION,
            ca_certificate,
            authentication: KafkaAuthenticationContract::SaslTls {
                mechanism,
                username,
                password,
            },
            replication_factor: 3,
            min_insync_replicas: 2,
            producer_acks_all: true,
            producer_idempotence: true,
            consumer_auto_commit: false,
            store_offset_after_apply: true,
            synchronous_offset_commit: true,
        }
    }

    pub fn validate(&self) -> Result<(), KafkaStreamError> {
        if self.contract_version != KAFKA_PRODUCTION_CONTRACT_VERSION {
            return Err(KafkaStreamError::UnsupportedProductionContractVersion {
                expected: KAFKA_PRODUCTION_CONTRACT_VERSION,
                actual: self.contract_version,
            });
        }
        self.ca_certificate.validate("ca_certificate")?;
        self.authentication.validate()?;
        if self.replication_factor < 3 {
            return Err(invalid("replication_factor", "must be at least three"));
        }
        if self.min_insync_replicas < 2 || self.min_insync_replicas > self.replication_factor {
            return Err(invalid(
                "min_insync_replicas",
                "must be at least two and no greater than replication_factor",
            ));
        }
        require_enabled("producer_acks_all", self.producer_acks_all)?;
        require_enabled("producer_idempotence", self.producer_idempotence)?;
        if self.consumer_auto_commit {
            return Err(invalid("consumer_auto_commit", "must be disabled"));
        }
        require_enabled("store_offset_after_apply", self.store_offset_after_apply)?;
        require_enabled("synchronous_offset_commit", self.synchronous_offset_commit)
    }
}

fn require_enabled(field: &'static str, enabled: bool) -> Result<(), KafkaStreamError> {
    if enabled {
        Ok(())
    } else {
        Err(invalid(field, "must be enabled"))
    }
}

fn invalid(field: &'static str, reason: &'static str) -> KafkaStreamError {
    KafkaStreamError::InvalidProductionContract { field, reason }
}
