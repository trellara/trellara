use std::collections::BTreeSet;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use trellara_stream::{PublishAck, StreamError, StreamMessage, StreamPublisher};

use crate::kafka_message::owned_headers;
use crate::publisher_config::KafkaPublisherConfig;
use crate::publisher_outcome::{
    classify_publish_error, validate_kafka_message_for_publish, KafkaPublishFailure,
    KafkaPublishIdentity,
};
use crate::topology_snapshot::KafkaTopologySnapshot;
use crate::{KafkaStreamError, KafkaTopologyRequirement};

pub struct KafkaPublisher {
    producer: FutureProducer,
    queue_timeout: Duration,
    topology: Option<KafkaTopologyRequirement>,
    validated_topics: Mutex<BTreeSet<String>>,
}

impl KafkaPublisher {
    pub fn new(config: KafkaPublisherConfig) -> Result<Self, KafkaStreamError> {
        config.validate()?;
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", config.bootstrap_servers)
            .set("client.id", config.client_id)
            .set("acks", "all")
            .set(
                "message.timeout.ms",
                config.message_timeout.as_millis().to_string(),
            )
            .set(
                "delivery.timeout.ms",
                config.message_timeout.as_millis().to_string(),
            )
            .set("enable.gapless.guarantee", "true");

        if config.enable_idempotence {
            client_config
                .set("enable.idempotence", "true")
                .set("max.in.flight.requests.per.connection", "5");
        }
        if let Some(security) = &config.security {
            security.apply(&mut client_config)?;
        }

        Ok(Self {
            producer: client_config.create().map_err(|_| {
                KafkaStreamError::ClientInitialization {
                    client: "publisher",
                }
            })?,
            queue_timeout: config.queue_timeout,
            topology: config.topology,
            validated_topics: Mutex::new(BTreeSet::new()),
        })
    }

    pub fn cluster_id(&self, timeout: Duration) -> Result<String, KafkaStreamError> {
        self.producer
            .client()
            .fetch_cluster_id(Timeout::After(timeout))
            .ok_or(KafkaStreamError::MissingClusterId)
    }

    pub fn validate_topology(&self, topics: &[String]) -> Result<(), KafkaStreamError> {
        let requirement = self.topology.as_ref().ok_or_else(|| {
            invalid_config(
                "topology",
                "a topology requirement must be configured before validation",
            )
        })?;
        if topics.is_empty() {
            return Err(invalid_config("topics", "must not be empty"));
        }
        let metadata = self
            .producer
            .client()
            .fetch_metadata(None, Timeout::After(requirement.metadata_timeout))?;
        KafkaTopologySnapshot::from_metadata(&metadata, topics).validate(topics, requirement)
    }

    fn ensure_topic_topology(&self, topic: &str) -> Result<(), KafkaStreamError> {
        if self.topology.is_none() {
            return Ok(());
        }
        if self
            .validated_topics
            .lock()
            .map_err(|_| invalid_config("topology", "validation state is unavailable"))?
            .contains(topic)
        {
            return Ok(());
        }
        self.validate_topology(&[topic.to_string()])?;
        self.validated_topics
            .lock()
            .map_err(|_| invalid_config("topology", "validation state is unavailable"))?
            .insert(topic.to_string());
        Ok(())
    }

    pub async fn publish_reconcilable(
        &self,
        message: StreamMessage,
    ) -> Result<PublishAck, KafkaPublishFailure> {
        validate_kafka_message_for_publish(&message).map_err(|error| {
            KafkaPublishFailure::Rejected {
                reason: error.to_string(),
            }
        })?;
        self.ensure_topic_topology(&message.topic)
            .map_err(|error| KafkaPublishFailure::Rejected {
                reason: error.to_string(),
            })?;
        let identity = KafkaPublishIdentity::from_message(&message);
        let headers = owned_headers(&message);
        let payload = message.payload.to_vec();
        let key = message.key.clone();
        let mut record = FutureRecord::to(&message.topic)
            .payload(&payload)
            .key(&key)
            .headers(headers);

        if let Some(partition) = message.partition {
            record = record.partition(partition);
        }

        let delivery = self
            .producer
            .send(record, Timeout::After(self.queue_timeout))
            .await
            .map_err(|(error, _)| classify_publish_error(error, identity))?;

        Ok(PublishAck {
            topic: message.topic,
            partition: delivery.0,
            offset: delivery.1,
        })
    }
}

#[async_trait]
impl StreamPublisher for KafkaPublisher {
    async fn publish(&self, message: StreamMessage) -> Result<PublishAck, StreamError> {
        self.publish_reconcilable(message)
            .await
            .map_err(|error| StreamError::Publisher(error.to_string()))
    }
}

fn invalid_config(field: &'static str, reason: &'static str) -> KafkaStreamError {
    KafkaStreamError::InvalidConfiguration { field, reason }
}
