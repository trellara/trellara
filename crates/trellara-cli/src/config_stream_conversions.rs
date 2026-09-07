#[cfg(feature = "kafka")]
use trellara_stream_kafka::{KafkaConsumerConfig, KafkaPublisherConfig};
#[cfg(feature = "local-stream")]
use trellara_stream_local::{LocalConsumerConfig, LocalPublisherConfig};

#[cfg(feature = "kafka")]
use crate::kafka_consumer_topics;
use crate::local_stream_topics;
#[cfg(feature = "kafka")]
use crate::KafkaProfile;
use crate::{replay_redelivery_topics, Result, TrellaraConfig};
#[cfg(any(feature = "kafka", feature = "local-stream"))]
use crate::{CliError, StreamConfig};

impl TrellaraConfig {
    #[cfg(feature = "kafka")]
    pub fn to_kafka_consumer_config(&self) -> Result<KafkaConsumerConfig> {
        self.validate()?;
        match &self.stream {
            StreamConfig::Kafka {
                bootstrap_servers,
                consumer_group,
                profile,
                ..
            } => {
                let group = consumer_group
                    .clone()
                    .unwrap_or_else(|| self.applier_consumer_group());
                let topics = kafka_consumer_topics(self)?;
                let mut config = match profile {
                    KafkaProfile::Development => {
                        KafkaConsumerConfig::local(bootstrap_servers, group, topics)
                    }
                    KafkaProfile::Production { contract } => {
                        KafkaConsumerConfig::production(bootstrap_servers, group, topics, contract)?
                    }
                };
                config.client_id =
                    format!("trellara-applier-{}-{}", self.source.id, self.dataset.id);
                Ok(config)
            }
            StreamConfig::Local { .. } => Err(CliError::InvalidConfig(
                "stream.kind must be kafka for Kafka consumer config".to_string(),
            )),
        }
    }

    #[cfg(feature = "local-stream")]
    pub fn to_local_consumer_config(&self) -> Result<LocalConsumerConfig> {
        self.validate()?;
        match &self.stream {
            StreamConfig::Local {
                path,
                consumer_group,
                durability,
            } => Ok(LocalConsumerConfig::new(
                path.clone(),
                consumer_group
                    .clone()
                    .unwrap_or_else(|| self.applier_consumer_group()),
                local_stream_topics(self)?,
            )
            .with_durability((*durability).into())),
            StreamConfig::Kafka { .. } => Err(CliError::InvalidConfig(
                "stream.kind must be local for local consumer config".to_string(),
            )),
        }
    }

    pub fn replay_redelivery_topics(&self) -> Result<Vec<String>> {
        self.validate()?;
        replay_redelivery_topics(self)
    }

    #[cfg(feature = "kafka")]
    pub fn to_kafka_publisher_config(&self) -> Result<KafkaPublisherConfig> {
        self.validate()?;
        match &self.stream {
            StreamConfig::Kafka {
                bootstrap_servers,
                profile,
                ..
            } => {
                let client_id = format!("trellara-relay-{}-{}", self.source.id, self.dataset.id);
                match profile {
                    KafkaProfile::Development => {
                        let mut config = KafkaPublisherConfig::local(bootstrap_servers);
                        config.client_id = client_id;
                        Ok(config)
                    }
                    KafkaProfile::Production { contract } => Ok(KafkaPublisherConfig::production(
                        bootstrap_servers,
                        client_id,
                        contract,
                    )?),
                }
            }
            StreamConfig::Local { .. } => Err(CliError::InvalidConfig(
                "stream.kind must be kafka for Kafka publisher config".to_string(),
            )),
        }
    }

    #[cfg(feature = "local-stream")]
    pub fn to_local_publisher_config(&self) -> Result<LocalPublisherConfig> {
        self.validate()?;
        match &self.stream {
            StreamConfig::Local {
                path, durability, ..
            } => Ok(LocalPublisherConfig::new(path.clone()).with_durability((*durability).into())),
            StreamConfig::Kafka { .. } => Err(CliError::InvalidConfig(
                "stream.kind must be local for local publisher config".to_string(),
            )),
        }
    }

    pub(crate) fn local_stream_topics(&self) -> Result<Vec<String>> {
        local_stream_topics(self)
    }

    fn applier_consumer_group(&self) -> String {
        format!("trellara-applier-{}-{}", self.source.id, self.dataset.id)
    }
}
