use super::types::{RuntimeQueueState, RuntimeQueueStatus};

pub(super) fn from_state(capacity: usize, queue: &RuntimeQueueState) -> RuntimeQueueStatus {
    RuntimeQueueStatus {
        preloaded: true,
        configured_capacity: capacity,
        queued_frames: usize::try_from(queue.len).expect("queue length fits usize"),
        enqueued_frames: queue.enqueued_frames,
        durable_frames: queue.durable_frames,
        duplicate_frames: queue.duplicate_frames,
        backpressure_events: queue.backpressure_events,
        relay_failures: queue.relay_failures,
        worker_starts: queue.worker_starts,
        worker_pid: queue.worker_pid,
        last_durable_lsn: format_lsn(queue.last_durable_lsn),
    }
}

pub(super) fn empty(capacity: usize) -> RuntimeQueueStatus {
    RuntimeQueueStatus {
        preloaded: false,
        configured_capacity: capacity,
        queued_frames: 0,
        enqueued_frames: 0,
        durable_frames: 0,
        duplicate_frames: 0,
        backpressure_events: 0,
        relay_failures: 0,
        worker_starts: 0,
        worker_pid: 0,
        last_durable_lsn: "0/0".to_string(),
    }
}

pub(crate) fn format_lsn(lsn: u64) -> String {
    format!("{:X}/{:X}", lsn >> 32, lsn & 0xffff_ffff)
}
