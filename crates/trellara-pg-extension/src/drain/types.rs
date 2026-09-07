use serde::Serialize;

use crate::NativeHandoffFrame;

pub const RELAY_HANDOFF_CONTRACT: &str =
    "background_worker_drains_ordered_frames_to_relay_before_source_ack";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeHandoffDrainBatch {
    pub contract: &'static str,
    pub frames: Vec<NativeHandoffFrame>,
    pub frame_count: usize,
    pub payload_bytes: usize,
    pub first_commit_lsn: String,
    pub last_commit_lsn: String,
    pub source_acknowledgement: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeHandoffDrainError {
    EmptyBatch,
    BatchExceedsQueueCapacity {
        frame_count: usize,
        capacity_frames: usize,
    },
    BatchPayloadExceedsQueueCapacity {
        payload_bytes: usize,
        max_buffered_payload_bytes: usize,
    },
    InvalidBoundaryField {
        field: &'static str,
        reason: String,
    },
    CommitLsnOrderRegression {
        previous_commit_lsn: String,
        current_commit_lsn: String,
    },
    DuplicateTransactionBoundary {
        boundary: String,
    },
}
