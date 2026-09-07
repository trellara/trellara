use std::collections::HashSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct StrictChunkTracker {
    published_chunks: HashSet<u32>,
    duplicate_chunks: usize,
}

impl StrictChunkTracker {
    pub(crate) fn publish(&mut self, chunk_id: u32) {
        if !self.published_chunks.insert(chunk_id) {
            self.duplicate_chunks += 1;
        }
    }

    pub(crate) fn published_count(&self) -> usize {
        self.published_chunks.len()
    }

    pub(crate) fn duplicate_count(&self) -> usize {
        self.duplicate_chunks
    }

    pub(crate) fn is_complete(&self, chunk_count: usize) -> bool {
        self.published_chunks.len() == chunk_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_unique_and_duplicate_chunk_publishes() {
        let mut tracker = StrictChunkTracker::default();

        tracker.publish(0);
        tracker.publish(1);
        tracker.publish(1);

        assert_eq!(tracker.published_count(), 2);
        assert_eq!(tracker.duplicate_count(), 1);
    }

    #[test]
    fn reports_complete_only_after_every_chunk_arrives() {
        let mut tracker = StrictChunkTracker::default();

        tracker.publish(0);
        tracker.publish(2);
        assert!(!tracker.is_complete(3));

        tracker.publish(1);
        assert!(tracker.is_complete(3));
    }
}
