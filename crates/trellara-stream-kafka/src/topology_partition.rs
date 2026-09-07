use crate::topology_snapshot::{invalid_topology, KafkaPartitionSnapshot, KafkaTopologySnapshot};
use crate::{KafkaStreamError, KafkaTopologyRequirement};

impl KafkaPartitionSnapshot {
    pub(crate) fn validate(
        &self,
        topic_name: &str,
        snapshot: &KafkaTopologySnapshot,
        requirement: &KafkaTopologyRequirement,
    ) -> Result<(), KafkaStreamError> {
        self.validate_liveness(topic_name)?;
        self.validate_replication(topic_name, requirement)?;
        self.validate_broker_membership(topic_name, snapshot)
    }

    fn validate_liveness(&self, topic_name: &str) -> Result<(), KafkaStreamError> {
        if self.error || self.leader < 0 {
            return Err(invalid_topology(format!(
                "topic {topic_name:?} partition {} has no healthy leader",
                self.id
            )));
        }
        Ok(())
    }

    fn validate_replication(
        &self,
        topic_name: &str,
        requirement: &KafkaTopologyRequirement,
    ) -> Result<(), KafkaStreamError> {
        if self.replicas.len() < requirement.minimum_replication_factor {
            return Err(invalid_topology(format!(
                "topic {topic_name:?} partition {} has {} replicas; at least {} required",
                self.id,
                self.replicas.len(),
                requirement.minimum_replication_factor
            )));
        }
        if self.in_sync_replicas.len() < requirement.minimum_in_sync_replicas {
            return Err(invalid_topology(format!(
                "topic {topic_name:?} partition {} has {} in-sync replicas; at least {} required",
                self.id,
                self.in_sync_replicas.len(),
                requirement.minimum_in_sync_replicas
            )));
        }
        Ok(())
    }

    fn validate_broker_membership(
        &self,
        topic_name: &str,
        snapshot: &KafkaTopologySnapshot,
    ) -> Result<(), KafkaStreamError> {
        if !snapshot.broker_ids.is_superset(&self.replicas) {
            return Err(invalid_topology(format!(
                "topic {topic_name:?} partition {} reports a replica outside the broker set",
                self.id
            )));
        }
        if !self.replicas.contains(&self.leader) {
            return Err(invalid_topology(format!(
                "topic {topic_name:?} partition {} reports a leader outside its replica set",
                self.id
            )));
        }
        if !self.replicas.is_superset(&self.in_sync_replicas) {
            return Err(invalid_topology(format!(
                "topic {topic_name:?} partition {} reports an ISR outside its replica set",
                self.id
            )));
        }
        Ok(())
    }
}
