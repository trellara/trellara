use std::fmt;

use rdkafka::error::{KafkaError, RDKafkaErrorCode};
use sha2::{Digest, Sha256};
use trellara_stream::{validate_message_shape, StreamError, StreamMessage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KafkaPublishIdentity {
    pub topic: String,
    pub partition: Option<i32>,
    pub key: String,
    pub content_sha256: String,
}

impl KafkaPublishIdentity {
    #[must_use]
    pub fn from_message(message: &StreamMessage) -> Self {
        let mut digest = Sha256::new();
        update_digest(&mut digest, message.key.as_bytes());
        update_digest(&mut digest, &message.payload);
        for header in &message.headers {
            update_digest(&mut digest, header.key.as_bytes());
            update_digest(&mut digest, header.value.as_bytes());
        }
        Self {
            topic: message.topic.clone(),
            partition: message
                .partition
                .or_else(|| message.position.as_ref().map(|position| position.partition)),
            key: message.key.clone(),
            content_sha256: format!("{:x}", digest.finalize()),
        }
    }

    #[must_use]
    pub fn matches(&self, observed: &StreamMessage) -> bool {
        let observed = Self::from_message(observed);
        self.topic == observed.topic
            && self.key == observed.key
            && self.content_sha256 == observed.content_sha256
            && self
                .partition
                .is_none_or(|partition| observed.partition == Some(partition))
    }
}

impl fmt::Display for KafkaPublishIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "topic {:?}, partition {:?}, content sha256 {}",
            self.topic, self.partition, self.content_sha256
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum KafkaPublishFailure {
    #[error("Kafka rejected the publish before a durable acknowledgement: {reason}")]
    Rejected { reason: String },
    #[error(
        "Kafka publish outcome is ambiguous for {identity} (broker error: {broker_error:?}); reconcile by identity before advancing source progress"
    )]
    Ambiguous {
        identity: KafkaPublishIdentity,
        broker_error: Option<RDKafkaErrorCode>,
    },
}

pub(crate) fn classify_publish_error(
    error: KafkaError,
    identity: KafkaPublishIdentity,
) -> KafkaPublishFailure {
    let code = error.rdkafka_error_code();
    if code.is_some_and(definitively_rejected) {
        KafkaPublishFailure::Rejected {
            reason: code.map_or_else(
                || "Kafka rejected the record".to_string(),
                |code| code.to_string(),
            ),
        }
    } else {
        KafkaPublishFailure::Ambiguous {
            identity,
            broker_error: code,
        }
    }
}

pub(crate) fn validate_kafka_message_for_publish(
    message: &StreamMessage,
) -> Result<(), StreamError> {
    validate_message_shape(message)
}

fn update_digest(digest: &mut Sha256, bytes: &[u8]) {
    digest.update(bytes.len().to_be_bytes());
    digest.update(bytes);
}

fn definitively_rejected(code: RDKafkaErrorCode) -> bool {
    matches!(
        code,
        RDKafkaErrorCode::QueueFull
            | RDKafkaErrorCode::TimedOutQueue
            | RDKafkaErrorCode::InvalidArgument
            | RDKafkaErrorCode::InvalidMessage
            | RDKafkaErrorCode::InvalidMessageSize
            | RDKafkaErrorCode::MessageSizeTooLarge
            | RDKafkaErrorCode::MessageBatchTooLarge
            | RDKafkaErrorCode::InvalidTopic
            | RDKafkaErrorCode::Authentication
            | RDKafkaErrorCode::UnsupportedFeature
            | RDKafkaErrorCode::KeySerialization
            | RDKafkaErrorCode::ValueSerialization
    )
}
