#[cfg(feature = "runtime")]
use rdkafka::error::KafkaError;
use thiserror::Error;

#[cfg(feature = "runtime")]
mod consumer;
#[cfg(feature = "runtime")]
mod consumer_config;
#[cfg(feature = "runtime")]
mod consumer_context;
#[cfg(feature = "runtime")]
mod consumer_progress;
#[cfg(feature = "runtime")]
mod kafka_ack_offsets;
#[cfg(feature = "runtime")]
mod kafka_message;
mod production_contract;
#[cfg(feature = "runtime")]
mod publisher;
#[cfg(feature = "runtime")]
mod publisher_config;
#[cfg(feature = "runtime")]
mod publisher_outcome;
#[cfg(feature = "runtime")]
mod security;
#[cfg(feature = "runtime")]
mod topology;
#[cfg(feature = "runtime")]
mod topology_partition;
#[cfg(feature = "runtime")]
mod topology_snapshot;

#[cfg(feature = "runtime")]
pub use consumer::KafkaConsumer;
#[cfg(feature = "runtime")]
pub use consumer_config::KafkaConsumerConfig;
pub use production_contract::{
    KafkaAuthenticationContract, KafkaProductionContract, KafkaSaslMechanism, KafkaSecretRef,
    KAFKA_PRODUCTION_CONTRACT_VERSION,
};
#[cfg(feature = "runtime")]
pub use publisher::KafkaPublisher;
#[cfg(feature = "runtime")]
pub use publisher_config::KafkaPublisherConfig;
#[cfg(feature = "runtime")]
pub use publisher_outcome::{KafkaPublishFailure, KafkaPublishIdentity};
#[cfg(feature = "runtime")]
pub use security::KafkaSecurityConfig;
#[cfg(feature = "runtime")]
pub use topology::KafkaTopologyRequirement;

#[derive(Debug, Error)]
pub enum KafkaStreamError {
    #[cfg(feature = "runtime")]
    #[error("kafka error: {0}")]
    Kafka(#[from] KafkaError),
    #[error("kafka consumer requires at least one topic")]
    MissingTopics,
    #[error("kafka broker did not return a cluster id")]
    MissingClusterId,
    #[error("unsupported Kafka production contract version {actual}; expected {expected}")]
    UnsupportedProductionContractVersion { expected: u16, actual: u16 },
    #[error("Kafka production contract field {field} is invalid: {reason}")]
    InvalidProductionContract {
        field: &'static str,
        reason: &'static str,
    },
    #[error("invalid kafka stream position {field}={value}: {reason}")]
    InvalidStreamPosition {
        field: &'static str,
        value: i64,
        reason: &'static str,
    },
    #[error("Kafka configuration field {field} is invalid: {reason}")]
    InvalidConfiguration {
        field: &'static str,
        reason: &'static str,
    },
    #[error("Kafka secret reference for {field} could not be resolved: {reason}")]
    SecretUnavailable {
        field: &'static str,
        reason: &'static str,
    },
    #[error("Kafka broker topology does not satisfy the configured durability quorum: {0}")]
    InvalidBrokerTopology(String),
    #[error("Kafka {client} client could not be initialized; configuration details were redacted")]
    ClientInitialization { client: &'static str },
    #[error("Kafka consumer no longer owns {topic} partition {partition}")]
    PartitionNotOwned { topic: String, partition: i32 },
    #[error("Kafka consumer cannot acknowledge undelivered offset {topic}[{partition}]@{offset}")]
    OffsetNotDelivered {
        topic: String,
        partition: i32,
        offset: i64,
    },
}

#[cfg(all(test, feature = "runtime"))]
#[path = "tests/mod.rs"]
mod tests;
