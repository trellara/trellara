use std::collections::BTreeSet;

use rdkafka::metadata::Metadata;

use crate::{KafkaStreamError, KafkaTopologyRequirement};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KafkaTopologySnapshot {
    pub(crate) broker_ids: BTreeSet<i32>,
    pub(crate) topics: Vec<KafkaTopicSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KafkaTopicSnapshot {
    pub(crate) name: String,
    pub(crate) error: bool,
    pub(crate) partitions: Vec<KafkaPartitionSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KafkaPartitionSnapshot {
    pub(crate) id: i32,
    pub(crate) leader: i32,
    pub(crate) replicas: BTreeSet<i32>,
    pub(crate) in_sync_replicas: BTreeSet<i32>,
    pub(crate) error: bool,
}

impl KafkaTopologySnapshot {
    pub(crate) fn from_metadata(metadata: &Metadata, topics: &[String]) -> Self {
        let requested = topics.iter().map(String::as_str).collect::<BTreeSet<_>>();
        Self {
            broker_ids: metadata
                .brokers()
                .iter()
                .map(|broker| broker.id())
                .collect(),
            topics: metadata
                .topics()
                .iter()
                .filter(|topic| requested.contains(topic.name()))
                .map(KafkaTopicSnapshot::from_metadata)
                .collect(),
        }
    }

    pub(crate) fn validate(
        &self,
        topics: &[String],
        requirement: &KafkaTopologyRequirement,
    ) -> Result<(), KafkaStreamError> {
        requirement.validate()?;
        self.validate_broker_count(requirement)?;
        for topic_name in topics {
            let topic = self.required_topic(topic_name)?;
            topic.validate(topic_name, self, requirement)?;
        }
        Ok(())
    }

    fn validate_broker_count(
        &self,
        requirement: &KafkaTopologyRequirement,
    ) -> Result<(), KafkaStreamError> {
        if self.broker_ids.len() >= requirement.minimum_broker_count {
            return Ok(());
        }
        Err(invalid_topology(format!(
            "cluster exposes {} distinct brokers; at least {} required",
            self.broker_ids.len(),
            requirement.minimum_broker_count
        )))
    }

    fn required_topic(&self, topic_name: &str) -> Result<&KafkaTopicSnapshot, KafkaStreamError> {
        self.topics
            .iter()
            .find(|topic| topic.name == topic_name)
            .ok_or_else(|| {
                invalid_topology(format!(
                    "required topic {topic_name:?} is absent from broker metadata"
                ))
            })
    }
}

impl KafkaTopicSnapshot {
    fn from_metadata(topic: &rdkafka::metadata::MetadataTopic) -> Self {
        Self {
            name: topic.name().to_string(),
            error: topic.error().is_some(),
            partitions: topic
                .partitions()
                .iter()
                .map(KafkaPartitionSnapshot::from_metadata)
                .collect(),
        }
    }

    fn validate(
        &self,
        topic_name: &str,
        snapshot: &KafkaTopologySnapshot,
        requirement: &KafkaTopologyRequirement,
    ) -> Result<(), KafkaStreamError> {
        if self.error || self.partitions.is_empty() {
            return Err(invalid_topology(format!(
                "required topic {topic_name:?} is unavailable or has no partitions"
            )));
        }
        for partition in &self.partitions {
            partition.validate(topic_name, snapshot, requirement)?;
        }
        Ok(())
    }
}

impl KafkaPartitionSnapshot {
    fn from_metadata(partition: &rdkafka::metadata::MetadataPartition) -> Self {
        Self {
            id: partition.id(),
            leader: partition.leader(),
            replicas: partition.replicas().iter().copied().collect(),
            in_sync_replicas: partition.isr().iter().copied().collect(),
            error: partition.error().is_some(),
        }
    }
}

pub(crate) fn invalid_topology(message: String) -> KafkaStreamError {
    KafkaStreamError::InvalidBrokerTopology(message)
}
