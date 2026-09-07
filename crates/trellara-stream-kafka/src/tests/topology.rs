use std::collections::BTreeSet;
use std::time::Duration;

use crate::topology_snapshot::{KafkaPartitionSnapshot, KafkaTopicSnapshot, KafkaTopologySnapshot};

use super::*;

fn requirement() -> KafkaTopologyRequirement {
    KafkaTopologyRequirement {
        minimum_broker_count: 3,
        minimum_replication_factor: 3,
        minimum_in_sync_replicas: 2,
        metadata_timeout: Duration::from_secs(1),
    }
}

fn topology(replicas: &[i32], isr: &[i32]) -> KafkaTopologySnapshot {
    KafkaTopologySnapshot {
        broker_ids: BTreeSet::from([1, 2, 3]),
        topics: vec![KafkaTopicSnapshot {
            name: "events".to_string(),
            error: false,
            partitions: vec![KafkaPartitionSnapshot {
                id: 0,
                leader: 1,
                replicas: replicas.iter().copied().collect(),
                in_sync_replicas: isr.iter().copied().collect(),
                error: false,
            }],
        }],
    }
}

#[test]
fn accepts_three_broker_two_isr_quorum() {
    topology(&[1, 2, 3], &[1, 2])
        .validate(&["events".to_string()], &requirement())
        .expect("healthy quorum");
}

#[test]
fn rejects_degraded_isr_even_when_replication_factor_is_three() {
    let error = topology(&[1, 2, 3], &[1])
        .validate(&["events".to_string()], &requirement())
        .expect_err("degraded ISR");

    assert!(error.to_string().contains("1 in-sync replicas"));
}

#[test]
fn rejects_replica_ids_that_are_not_distinct() {
    let error = topology(&[1, 1, 2], &[1, 2])
        .validate(&["events".to_string()], &requirement())
        .expect_err("duplicate replicas do not form a quorum");

    assert!(error.to_string().contains("2 replicas"));
}

#[test]
fn rejects_isr_outside_replica_set() {
    let error = topology(&[1, 2, 3], &[1, 4])
        .validate(&["events".to_string()], &requirement())
        .expect_err("invalid ISR");

    assert!(error.to_string().contains("outside its replica set"));
}

#[test]
fn rejects_replica_not_present_in_broker_metadata() {
    let error = topology(&[1, 2, 4], &[1, 2])
        .validate(&["events".to_string()], &requirement())
        .expect_err("unknown replica broker");

    assert!(error.to_string().contains("outside the broker set"));
}

#[test]
fn rejects_missing_required_topic() {
    let error = topology(&[1, 2, 3], &[1, 2])
        .validate(&["missing".to_string()], &requirement())
        .expect_err("missing required topic");

    assert!(error.to_string().contains("absent from broker metadata"));
}
