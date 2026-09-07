use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{KafkaProductionContract, KafkaStreamError};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KafkaTopologyRequirement {
    pub minimum_broker_count: usize,
    pub minimum_replication_factor: usize,
    pub minimum_in_sync_replicas: usize,
    pub metadata_timeout: Duration,
}

impl KafkaTopologyRequirement {
    #[must_use]
    pub fn from_production_contract(contract: &KafkaProductionContract) -> Self {
        Self {
            minimum_broker_count: usize::from(contract.replication_factor),
            minimum_replication_factor: usize::from(contract.replication_factor),
            minimum_in_sync_replicas: usize::from(contract.min_insync_replicas),
            metadata_timeout: Duration::from_secs(10),
        }
    }

    pub fn validate(&self) -> Result<(), KafkaStreamError> {
        if self.minimum_broker_count == 0 {
            return Err(invalid_requirement("minimum_broker_count"));
        }
        if self.minimum_replication_factor == 0 {
            return Err(invalid_requirement("minimum_replication_factor"));
        }
        if self.minimum_in_sync_replicas == 0
            || self.minimum_in_sync_replicas > self.minimum_replication_factor
        {
            return Err(KafkaStreamError::InvalidConfiguration {
                field: "topology.minimum_in_sync_replicas",
                reason: "must be non-zero and no greater than minimum_replication_factor",
            });
        }
        if self.minimum_broker_count < self.minimum_replication_factor {
            return Err(KafkaStreamError::InvalidConfiguration {
                field: "topology.minimum_broker_count",
                reason: "must be at least minimum_replication_factor",
            });
        }
        if self.metadata_timeout.is_zero() {
            return Err(KafkaStreamError::InvalidConfiguration {
                field: "topology.metadata_timeout",
                reason: "must be greater than zero",
            });
        }
        Ok(())
    }
}

fn invalid_requirement(field: &'static str) -> KafkaStreamError {
    KafkaStreamError::InvalidConfiguration {
        field,
        reason: "must be greater than zero",
    }
}
