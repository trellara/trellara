use serde::Serialize;

use crate::{NativeHandoffFrame, MAX_HANDOFF_PAYLOAD_BYTES};

pub const DEFAULT_HANDOFF_QUEUE_FRAMES: usize = 1024;
pub const MAX_HANDOFF_QUEUE_FRAMES: usize = 65_536;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeHandoffQueueConfig {
    pub capacity_frames: usize,
    pub max_frame_payload_bytes: usize,
    pub max_buffered_payload_bytes: usize,
    pub overflow_policy: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeHandoffQueueAdmission {
    pub queued_frames_after: usize,
    pub queued_payload_bytes_after: usize,
    pub source_acknowledgement: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeHandoffQueueError {
    InvalidFrameCapacity,
    InvalidFramePayloadLimit,
    BufferedPayloadLimitTooSmall {
        max_frame_payload_bytes: usize,
        max_buffered_payload_bytes: usize,
    },
    FrameCapacityExceeded {
        queued_frames: usize,
        capacity_frames: usize,
    },
    BufferedPayloadCapacityExceeded {
        queued_payload_bytes: usize,
        frame_payload_bytes: usize,
        max_buffered_payload_bytes: usize,
    },
}

pub fn native_handoff_queue_config(
    capacity_frames: usize,
    max_frame_payload_bytes: usize,
    max_buffered_payload_bytes: usize,
) -> Result<NativeHandoffQueueConfig, NativeHandoffQueueError> {
    if capacity_frames == 0 || capacity_frames > MAX_HANDOFF_QUEUE_FRAMES {
        return Err(NativeHandoffQueueError::InvalidFrameCapacity);
    }
    if max_frame_payload_bytes == 0 || max_frame_payload_bytes > MAX_HANDOFF_PAYLOAD_BYTES {
        return Err(NativeHandoffQueueError::InvalidFramePayloadLimit);
    }
    if max_buffered_payload_bytes < max_frame_payload_bytes {
        return Err(NativeHandoffQueueError::BufferedPayloadLimitTooSmall {
            max_frame_payload_bytes,
            max_buffered_payload_bytes,
        });
    }

    Ok(NativeHandoffQueueConfig {
        capacity_frames,
        max_frame_payload_bytes,
        max_buffered_payload_bytes,
        overflow_policy: "block_source_backend_until_relay_drains",
    })
}

pub fn native_handoff_queue_admission(
    config: &NativeHandoffQueueConfig,
    queued_frames: usize,
    queued_payload_bytes: usize,
    frame: &NativeHandoffFrame,
) -> Result<NativeHandoffQueueAdmission, NativeHandoffQueueError> {
    if queued_frames >= config.capacity_frames {
        return Err(NativeHandoffQueueError::FrameCapacityExceeded {
            queued_frames,
            capacity_frames: config.capacity_frames,
        });
    }

    let queued_payload_bytes_after = queued_payload_bytes
        .checked_add(frame.payload_bytes)
        .ok_or(NativeHandoffQueueError::BufferedPayloadCapacityExceeded {
            queued_payload_bytes,
            frame_payload_bytes: frame.payload_bytes,
            max_buffered_payload_bytes: config.max_buffered_payload_bytes,
        })?;
    if queued_payload_bytes_after > config.max_buffered_payload_bytes {
        return Err(NativeHandoffQueueError::BufferedPayloadCapacityExceeded {
            queued_payload_bytes,
            frame_payload_bytes: frame.payload_bytes,
            max_buffered_payload_bytes: config.max_buffered_payload_bytes,
        });
    }

    Ok(NativeHandoffQueueAdmission {
        queued_frames_after: queued_frames + 1,
        queued_payload_bytes_after,
        source_acknowledgement: crate::SOURCE_ACKNOWLEDGEMENT_CONTRACT,
    })
}
