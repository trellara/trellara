use std::time::Duration;

use tokio::runtime::{Builder, Runtime};
use trellara_pg_extension::NativeRelayWireFrame;
use trellara_stream::{StreamPublisher, TopicLayout};
use trellara_stream_kafka::{KafkaPublisher, KafkaPublisherConfig};

#[path = "native_kafka_message.rs"]
mod native_kafka_message;

use crate::native_publish_proof::{NativeKafkaPublishProof, NativePublishDestination};
use crate::native_transport::{NativeRelayTransportError, NativeRelayTransportResult};
use native_kafka_message::message_for_frame;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeKafkaRelayConfig {
    pub bootstrap_servers: String,
    pub client_id: String,
    pub message_timeout: Duration,
    pub proof_path: std::path::PathBuf,
}

impl NativeKafkaRelayConfig {
    pub fn local(
        bootstrap_servers: impl Into<String>,
        proof_path: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            bootstrap_servers: bootstrap_servers.into(),
            client_id: "trellara-native-relay".to_string(),
            message_timeout: Duration::from_secs(30),
            proof_path: proof_path.into(),
        }
    }
}

pub(crate) trait NativeFramePublisher {
    fn destination(
        &self,
        frame: &NativeRelayWireFrame,
    ) -> NativeRelayTransportResult<NativePublishDestination>;

    fn publish(
        &self,
        frame: &NativeRelayWireFrame,
    ) -> NativeRelayTransportResult<NativeKafkaPublishProof>;
}

pub(crate) struct NativeKafkaPublisher {
    runtime: Runtime,
    publisher: KafkaPublisher,
    cluster_id: String,
}

impl NativeKafkaPublisher {
    pub(crate) fn new(config: &NativeKafkaRelayConfig) -> NativeRelayTransportResult<Self> {
        if config.bootstrap_servers.trim().is_empty() {
            return Err(NativeRelayTransportError::KafkaConfiguration(
                "bootstrap servers cannot be empty".to_string(),
            ));
        }
        if config.message_timeout.is_zero() {
            return Err(NativeRelayTransportError::KafkaConfiguration(
                "message timeout must be greater than zero".to_string(),
            ));
        }
        let mut publisher_config = KafkaPublisherConfig::local(&config.bootstrap_servers);
        publisher_config.client_id.clone_from(&config.client_id);
        publisher_config.message_timeout = config.message_timeout;
        let publisher = KafkaPublisher::new(publisher_config)
            .map_err(|error| NativeRelayTransportError::KafkaConfiguration(error.to_string()))?;
        let cluster_id = publisher
            .cluster_id(config.message_timeout)
            .map_err(|error| NativeRelayTransportError::KafkaConfiguration(error.to_string()))?;
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(NativeRelayTransportError::Io)?;
        Ok(Self {
            runtime,
            publisher,
            cluster_id,
        })
    }
}

impl NativeFramePublisher for NativeKafkaPublisher {
    fn destination(
        &self,
        frame: &NativeRelayWireFrame,
    ) -> NativeRelayTransportResult<NativePublishDestination> {
        let topic = TopicLayout::new(&frame.source_id, &frame.dataset_id)
            .map_err(|error| NativeRelayTransportError::KafkaPublish(error.to_string()))?
            .strict_topic();
        Ok(NativePublishDestination {
            cluster_id: self.cluster_id.clone(),
            topic,
            partition: 0,
        })
    }

    fn publish(
        &self,
        frame: &NativeRelayWireFrame,
    ) -> NativeRelayTransportResult<NativeKafkaPublishProof> {
        let destination = self.destination(frame)?;
        let message = message_for_frame(frame, &destination);
        let ack = self
            .runtime
            .block_on(self.publisher.publish(message))
            .map_err(|error| NativeRelayTransportError::KafkaPublish(error.to_string()))?;
        if ack.topic != destination.topic || ack.partition != destination.partition {
            return Err(NativeRelayTransportError::KafkaDestinationMismatch);
        }
        NativeKafkaPublishProof::new(frame, destination, ack.offset)
    }
}
