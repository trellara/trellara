use std::collections::{BTreeMap, BTreeSet};

use rdkafka::{Offset, TopicPartitionList};
use trellara_stream::StreamPosition;

use crate::kafka_ack_offsets::ack_offsets;
use crate::KafkaStreamError;

type PartitionKey = (String, i32);

#[derive(Debug, Default)]
pub(crate) struct ConsumerProgress {
    assignments: BTreeSet<PartitionKey>,
    partitions: BTreeMap<PartitionKey, PartitionProgress>,
}

#[derive(Debug)]
struct PartitionProgress {
    next_commit_offset: i64,
    delivered: BTreeMap<i64, bool>,
}

#[derive(Debug)]
pub(crate) struct PendingCommit {
    pub(crate) offsets: TopicPartitionList,
    advances: Vec<(PartitionKey, i64)>,
}

impl ConsumerProgress {
    pub(crate) fn assign(&mut self, partitions: &TopicPartitionList) {
        for element in partitions.elements() {
            self.assignments
                .insert((element.topic().to_string(), element.partition()));
        }
    }

    pub(crate) fn revoke(&mut self, partitions: &TopicPartitionList) {
        for element in partitions.elements() {
            let key = (element.topic().to_string(), element.partition());
            self.assignments.remove(&key);
            self.partitions.remove(&key);
        }
    }

    pub(crate) fn record_delivery(
        &mut self,
        position: &StreamPosition,
    ) -> Result<(), KafkaStreamError> {
        ack_offsets(position)?;
        let key = (position.topic.clone(), position.partition);
        if !self.assignments.contains(&key) {
            return Err(KafkaStreamError::PartitionNotOwned {
                topic: position.topic.clone(),
                partition: position.partition,
            });
        }
        let progress = self.partitions.entry(key).or_insert(PartitionProgress {
            next_commit_offset: position.offset,
            delivered: BTreeMap::new(),
        });
        if position.offset < progress.next_commit_offset {
            return Ok(());
        }
        progress.delivered.entry(position.offset).or_insert(false);
        Ok(())
    }

    pub(crate) fn acknowledge(
        &mut self,
        position: &StreamPosition,
    ) -> Result<Option<PendingCommit>, KafkaStreamError> {
        ack_offsets(position)?;
        let key = (position.topic.clone(), position.partition);
        if !self.assignments.contains(&key) {
            return Err(KafkaStreamError::PartitionNotOwned {
                topic: position.topic.clone(),
                partition: position.partition,
            });
        }
        let Some(progress) = self.partitions.get_mut(&key) else {
            return Err(KafkaStreamError::OffsetNotDelivered {
                topic: position.topic.clone(),
                partition: position.partition,
                offset: position.offset,
            });
        };
        if position.offset < progress.next_commit_offset {
            return Ok(None);
        }
        let Some(applied) = progress.delivered.get_mut(&position.offset) else {
            return Err(KafkaStreamError::OffsetNotDelivered {
                topic: position.topic.clone(),
                partition: position.partition,
                offset: position.offset,
            });
        };
        *applied = true;
        Ok(self.pending_commit_for_keys(std::iter::once(key)))
    }

    pub(crate) fn pending_commit_for(
        &self,
        partitions: &TopicPartitionList,
    ) -> Option<PendingCommit> {
        self.pending_commit_for_keys(
            partitions
                .elements()
                .into_iter()
                .map(|element| (element.topic().to_string(), element.partition())),
        )
    }

    fn pending_commit_for_keys(
        &self,
        keys: impl IntoIterator<Item = PartitionKey>,
    ) -> Option<PendingCommit> {
        let mut offsets = TopicPartitionList::new();
        let mut advances = Vec::new();
        for key in keys {
            let Some(progress) = self.partitions.get(&key) else {
                continue;
            };
            let mut next = progress.next_commit_offset;
            while progress.delivered.get(&next).copied() == Some(true) {
                next += 1;
            }
            if next > progress.next_commit_offset {
                offsets
                    .add_partition_offset(&key.0, key.1, Offset::Offset(next))
                    .expect("validated topic and partition must form a commit offset");
                advances.push((key, next));
            }
        }
        if advances.is_empty() {
            None
        } else {
            Some(PendingCommit { offsets, advances })
        }
    }

    pub(crate) fn commit_succeeded(&mut self, pending: PendingCommit) {
        for (key, next) in pending.advances {
            if let Some(progress) = self.partitions.get_mut(&key) {
                progress.delivered.retain(|offset, _| *offset >= next);
                progress.next_commit_offset = next;
            }
        }
    }

    pub(crate) fn in_flight(&self) -> usize {
        self.partitions
            .values()
            .map(|progress| progress.delivered.len())
            .sum()
    }
}
