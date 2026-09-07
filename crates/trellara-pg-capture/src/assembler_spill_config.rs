use std::path::PathBuf;

use crate::assembler_buffer::PendingChangeBuffer;
use crate::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES;

#[derive(Clone, Debug)]
pub(crate) struct StreamSpillConfig {
    threshold_changes: usize,
    dir: Option<PathBuf>,
}

impl Default for StreamSpillConfig {
    fn default() -> Self {
        Self {
            threshold_changes: DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
            dir: None,
        }
    }
}

impl StreamSpillConfig {
    pub(crate) fn new(threshold_changes: usize, dir: Option<PathBuf>) -> Self {
        Self {
            threshold_changes,
            dir,
        }
    }

    pub(crate) fn pending_change_buffer(&self) -> PendingChangeBuffer {
        PendingChangeBuffer::memory_in(self.threshold_changes, self.dir.clone())
    }
}
