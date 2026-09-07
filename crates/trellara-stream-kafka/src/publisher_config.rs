use std::time::Duration;

use crate::{
    KafkaProductionContract, KafkaSecurityConfig, KafkaStreamError, KafkaTopologyRequirement,
};

#[derive(Clone, Debug)]
pub struct KafkaPublisherConfig {
    pub bootstrap_servers: String,
    pub client_id: String,
    pub message_timeout: Duration,
    pub queue_timeout: Duration,
    pub enable_idempotence: bool,
    pub security: Option<KafkaSecurityConfig>,
    pub topology: Option<KafkaTopologyRequirement>,
}

impl KafkaPublisherConfig {
    pub fn local(bootstrap_servers: impl Into<String>) -> Self {
        Self {
            bootstrap_servers: bootstrap_servers.into(),
            client_id: "trellara-relay".to_string(),
            message_timeout: Duration::from_secs(30),
            queue_timeout: Duration::from_secs(5),
            enable_idempotence: true,
            security: None,
            topology: None,
        }
    }

    pub fn production(
        bootstrap_servers: impl Into<String>,
        client_id: impl Into<String>,
        contract: &KafkaProductionContract,
    ) -> Result<Self, KafkaStreamError> {
        contract.validate()?;
        let mut config = Self::local(bootstrap_servers);
        config.client_id = client_id.into();
        config.security = Some(KafkaSecurityConfig::from_production_contract(contract));
        config.topology = Some(KafkaTopologyRequirement::from_production_contract(contract));
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), KafkaStreamError> {
        if self.bootstrap_servers.trim().is_empty() {
            return Err(invalid_config("bootstrap_servers", "must not be empty"));
        }
        if self.client_id.trim().is_empty() {
            return Err(invalid_config("client_id", "must not be empty"));
        }
        if self.message_timeout.is_zero() {
            return Err(invalid_config(
                "message_timeout",
                "must be greater than zero",
            ));
        }
        if self.queue_timeout.is_zero() {
            return Err(invalid_config("queue_timeout", "must be greater than zero"));
        }
        if self.topology.is_some() && !self.enable_idempotence {
            return Err(invalid_config(
                "enable_idempotence",
                "must be enabled when production topology validation is configured",
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
