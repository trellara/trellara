use std::fmt;

use pgrx::PGRXSharedMemory;
use serde::Serialize;

use crate::MAX_LOGICAL_FRAME_BYTES;

#[path = "types/frame.rs"]
mod frame;

pub(crate) use frame::RuntimeQueuedFrame;

pub(crate) const MAX_RUNTIME_QUEUE_FRAMES: usize = 16;

#[derive(Clone, Copy)]
#[repr(C)]
pub(super) struct RuntimeQueueState {
    pub(super) frames: [RuntimeQueuedFrame; MAX_RUNTIME_QUEUE_FRAMES],
    pub(super) head: u32,
    pub(super) len: u32,
    pub(super) enqueued_frames: u64,
    pub(super) durable_frames: u64,
    pub(super) duplicate_frames: u64,
    pub(super) backpressure_events: u64,
    pub(super) relay_failures: u64,
    pub(super) worker_starts: u64,
    pub(super) worker_pid: i32,
    pub(super) last_durable_lsn: u64,
    pub(super) last_durable_digest: [u8; 32],
}

impl Default for RuntimeQueueState {
    fn default() -> Self {
        Self {
            frames: [RuntimeQueuedFrame::EMPTY; MAX_RUNTIME_QUEUE_FRAMES],
            head: 0,
            len: 0,
            enqueued_frames: 0,
            durable_frames: 0,
            duplicate_frames: 0,
            backpressure_events: 0,
            relay_failures: 0,
            worker_starts: 0,
            worker_pid: 0,
            last_durable_lsn: 0,
            last_durable_digest: [0; 32],
        }
    }
}

unsafe impl PGRXSharedMemory for RuntimeQueueState {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RuntimeQueueError {
    NotPreloaded,
    Full { capacity: usize },
    IdentityTooLong { field: &'static str, length: usize },
    PayloadTooLarge(usize),
    CounterOverflow(&'static str),
}

impl fmt::Display for RuntimeQueueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPreloaded => formatter
                .write_str("trellara_pg_extension must be listed in shared_preload_libraries"),
            Self::Full { capacity } => {
                write!(
                    formatter,
                    "native shared-memory queue is full at {capacity} frames"
                )
            }
            Self::IdentityTooLong { field, length } => {
                write!(formatter, "{field} is {length} bytes; maximum is 255")
            }
            Self::PayloadTooLarge(length) => write!(
                formatter,
                "native payload is {length} bytes; maximum is {MAX_LOGICAL_FRAME_BYTES}"
            ),
            Self::CounterOverflow(field) => write!(formatter, "{field} counter overflowed"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct RuntimeQueueStatus {
    pub preloaded: bool,
    pub configured_capacity: usize,
    pub queued_frames: usize,
    pub enqueued_frames: u64,
    pub durable_frames: u64,
    pub duplicate_frames: u64,
    pub backpressure_events: u64,
    pub relay_failures: u64,
    pub worker_starts: u64,
    pub worker_pid: i32,
    pub last_durable_lsn: String,
}
