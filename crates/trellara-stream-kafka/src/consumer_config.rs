use std::time::Duration;

use crate::{
    KafkaProductionContract, KafkaSecurityConfig, KafkaStreamError, KafkaTopologyRequirement,
};

#[derive(Clone, Debug)]
pub struct KafkaConsumerConfig {
    pub bootstrap_servers: String,
    pub group_id: String,
    pub topics: Vec<String>,
    pub client_id: String,
    pub poll_timeout: Duration,
    pub enable_auto_commit: bool,
    pub max_in_flight_messages: usize,
    pub max_poll_interval: Duration,
    pub queued_max_messages_kbytes: usize,
    pub security: Option<KafkaSecurityConfig>,
    pub topology: Option<KafkaTopologyRequirement>,
}

impl KafkaConsumerConfig {
    pub fn local(
        bootstrap_servers: impl Into<String>,
        group_id: impl Into<String>,
        topics: Vec<String>,
    ) -> Self {
        Self {
            bootstrap_servers: bootstrap_servers.into(),
            group_id: group_id.into(),
            topics,
            client_id: "trellara-applier".to_string(),
            poll_timeout: Duration::from_millis(100),
            enable_auto_commit: false,
            max_in_flight_messages: 1_024,
            max_poll_interval: Duration::from_secs(300),
            queued_max_messages_kbytes: 64 * 1_024,
            security: None,
            topology: None,
        }
    }

    pub fn production(
        bootstrap_servers: impl Into<String>,
        group_id: impl Into<String>,
        topics: Vec<String>,
        contract: &KafkaProductionContract,
    ) -> Result<Self, KafkaStreamError> {
        contract.validate()?;
        let mut config = Self::local(bootstrap_servers, group_id, topics);
        config.security = Some(KafkaSecurityConfig::from_production_contract(contract));
        config.topology = Some(KafkaTopologyRequirement::from_production_contract(contract));
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), KafkaStreamError> {
        if self.bootstrap_servers.trim().is_empty() {
            return Err(invalid_config("bootstrap_servers", "must not be empty"));
        }
        if self.group_id.trim().is_empty() {
            return Err(invalid_config("group_id", "must not be empty"));
        }
        if self.client_id.trim().is_empty() {
            return Err(invalid_config("client_id", "must not be empty"));
        }
        if self.topics.is_empty() {
            return Err(KafkaStreamError::MissingTopics);
        }
        if self.topics.iter().any(|topic| topic.trim().is_empty()) {
            return Err(invalid_config("topics", "must not contain empty names"));
        }
        if self.enable_auto_commit {
            return Err(invalid_config(
                "enable_auto_commit",
                "must be disabled so progress follows target apply",
            ));
        }
        if self.poll_timeout.is_zero() {
            return Err(invalid_config("poll_timeout", "must be greater than zero"));
        }
        if self.max_in_flight_messages == 0 {
            return Err(invalid_config(
                "max_in_flight_messages",
                "must be greater than zero",
            ));
        }
        if self.max_poll_interval.is_zero() {
            return Err(invalid_config(
                "max_poll_interval",
                "must be greater than zero",
            ));
        }
        if self.queued_max_messages_kbytes == 0 {
            return Err(invalid_config(
                "queued_max_messages_kbytes",
                "must be greater than zero",
            ));
        }
        if let Some(topology) = &self.topology {
            topology.validate()?;
        }
        Ok(())
    }
}

fn invalid_config(field: &'static str, reason: &'static str) -> KafkaStreamError {
    KafkaStreamError::InvalidConfiguration { field, reason }
}
