use std::collections::BTreeMap;

use trellara_protocol::{PartitionChunk, TransactionCommitMarker, TransactionManifest};
use trellara_stream::StreamMessage;

use crate::barrier::HeaderContext;

#[derive(Default)]
pub(crate) struct PendingBarrierTransaction {
    pub(crate) manifest: Option<PendingManifest>,
    pub(crate) commit_marker: Option<PendingCommitMarker>,
    pub(crate) chunks: BTreeMap<u32, PendingChunk>,
}

impl PendingBarrierTransaction {
    pub(crate) fn expected_chunk_count(&self) -> u64 {
        self.manifest
            .as_ref()
            .map(|manifest| len_as_stat_count(manifest.manifest.partitions.len()))
            .unwrap_or_default()
    }

    pub(crate) fn buffered_chunk_count(&self) -> u64 {
        len_as_stat_count(self.chunks.len())
    }

    pub(crate) fn missing_chunk_count(&self) -> u64 {
        let Some(manifest) = &self.manifest else {
            return 0;
        };
        manifest
            .manifest
            .partitions
            .iter()
            .filter(|partition| !self.chunks.contains_key(&partition.id))
            .count()
            .try_into()
            .unwrap_or(u64::MAX)
    }

    pub(crate) fn extra_chunk_count(&self) -> u64 {
        let Some(manifest) = &self.manifest else {
            return 0;
        };
        self.chunks
            .keys()
            .filter(|partition_id| {
                !manifest
                    .manifest
                    .partitions
                    .iter()
                    .any(|partition| partition.id == **partition_id)
            })
            .count()
            .try_into()
            .unwrap_or(u64::MAX)
    }

    pub(crate) fn has_complete_chunk_set(&self) -> bool {
        let Some(manifest) = &self.manifest else {
            return false;
        };
        manifest
            .manifest
            .partitions
            .iter()
            .all(|partition| self.chunks.contains_key(&partition.id))
            && self.extra_chunk_count() == 0
    }

    pub(crate) fn has_invalid_commit_marker(&self) -> bool {
        let (Some(manifest), Some(marker)) = (&self.manifest, &self.commit_marker) else {
            return false;
        };
        !marker.marker.matches_manifest(&manifest.manifest)
    }

    pub(crate) fn buffered_message_count(&self) -> u64 {
        let chunk_messages = self
            .chunks
            .values()
            .map(|chunk| len_as_stat_count(chunk.messages.len()))
            .fold(0u64, u64::saturating_add);
        let manifest_messages = self
            .manifest
            .as_ref()
            .map(|manifest| len_as_stat_count(manifest.messages.len()))
            .unwrap_or_default();
        let commit_marker_messages = self
            .commit_marker
            .as_ref()
            .map(|marker| len_as_stat_count(marker.messages.len()))
            .unwrap_or_default();

        chunk_messages
            .saturating_add(manifest_messages)
            .saturating_add(commit_marker_messages)
    }
}

pub(crate) struct PendingManifest {
    pub(crate) manifest: TransactionManifest,
    pub(crate) context: HeaderContext,
    pub(crate) messages: Vec<StreamMessage>,
}

pub(crate) struct PendingCommitMarker {
    pub(crate) marker: TransactionCommitMarker,
    pub(crate) messages: Vec<StreamMessage>,
}

pub(crate) struct PendingChunk {
    pub(crate) chunk: PartitionChunk,
    pub(crate) messages: Vec<StreamMessage>,
}

pub(crate) fn same_partition_chunk(left: &PartitionChunk, right: &PartitionChunk) -> bool {
    left.transaction_id == right.transaction_id
        && left.partition_id == right.partition_id
        && left.checksum == right.checksum
        && left.compute_checksum() == right.compute_checksum()
}

fn len_as_stat_count(len: usize) -> u64 {
    len.try_into().unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn len_as_stat_count_accepts_normal_lengths() {
        assert_eq!(len_as_stat_count(42), 42);
    }
}
