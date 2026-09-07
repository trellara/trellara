use std::sync::atomic::{AtomicBool, Ordering};

use pgrx::{pg_guard, pg_shmem_init, pg_sys, PgLwLock};

use crate::NativeRelayWireFrame;

mod status;
mod types;

use types::RuntimeQueueState;
pub(crate) use types::{
    RuntimeQueueError, RuntimeQueueStatus, RuntimeQueuedFrame, MAX_RUNTIME_QUEUE_FRAMES,
};

static PRELOADED: AtomicBool = AtomicBool::new(false);
static RUNTIME_QUEUE: PgLwLock<RuntimeQueueState> =
    unsafe { PgLwLock::new(c"trellara_native_queue") };

#[allow(unexpected_cfgs)]
pub(crate) fn initialize() {
    PRELOADED.store(true, Ordering::Release);
    pg_shmem_init!(RUNTIME_QUEUE);
}

pub(crate) fn is_preloaded() -> bool {
    PRELOADED.load(Ordering::Acquire)
}

pub(crate) fn record_worker_start(pid: i32) -> Result<(), RuntimeQueueError> {
    require_preloaded()?;
    let mut queue = RUNTIME_QUEUE.exclusive();
    queue.worker_pid = pid;
    queue.worker_starts = checked_increment(queue.worker_starts, "worker_starts")?;
    Ok(())
}

pub(crate) fn has_capacity(capacity: usize) -> Result<bool, RuntimeQueueError> {
    require_preloaded()?;
    let mut queue = RUNTIME_QUEUE.exclusive();
    let has_capacity = usize::try_from(queue.len).expect("queue length fits usize") < capacity;
    if !has_capacity {
        queue.backpressure_events =
            checked_increment(queue.backpressure_events, "backpressure_events")?;
    }
    Ok(has_capacity)
}

pub(crate) fn enqueue(
    frame: &NativeRelayWireFrame,
    capacity: usize,
) -> Result<(), RuntimeQueueError> {
    require_preloaded()?;
    let queued = RuntimeQueuedFrame::from_wire(frame)?;
    let mut queue = RUNTIME_QUEUE.exclusive();
    if queue
        .frames
        .iter()
        .any(|existing| existing.matches(&queued))
        || (queue.last_durable_lsn >= queued.commit_lsn()
            && queue.last_durable_digest == queued.payload_digest())
    {
        queue.duplicate_frames = checked_increment(queue.duplicate_frames, "duplicate_frames")?;
        return Ok(());
    }
    let len = usize::try_from(queue.len).expect("queue length fits usize");
    if len >= capacity {
        queue.backpressure_events =
            checked_increment(queue.backpressure_events, "backpressure_events")?;
        return Err(RuntimeQueueError::Full { capacity });
    }
    let head = usize::try_from(queue.head).expect("queue head fits usize");
    let tail = (head + len) % MAX_RUNTIME_QUEUE_FRAMES;
    queue.frames[tail] = queued;
    queue.len += 1;
    queue.enqueued_frames = checked_increment(queue.enqueued_frames, "enqueued_frames")?;
    Ok(())
}

pub(crate) fn peek() -> Result<Option<RuntimeQueuedFrame>, RuntimeQueueError> {
    require_preloaded()?;
    let queue = RUNTIME_QUEUE.share();
    if queue.len == 0 {
        return Ok(None);
    }
    let head = usize::try_from(queue.head).expect("queue head fits usize");
    Ok(Some(queue.frames[head]))
}

pub(crate) fn mark_durable(frame: &RuntimeQueuedFrame) -> Result<(), RuntimeQueueError> {
    require_preloaded()?;
    let mut queue = RUNTIME_QUEUE.exclusive();
    if queue.len == 0 {
        return Ok(());
    }
    let head = usize::try_from(queue.head).expect("queue head fits usize");
    if !queue.frames[head].matches(frame) {
        return Ok(());
    }
    queue.last_durable_lsn = frame.commit_lsn();
    queue.last_durable_digest = frame.payload_digest();
    queue.durable_frames = checked_increment(queue.durable_frames, "durable_frames")?;
    queue.frames[head] = RuntimeQueuedFrame::EMPTY;
    queue.head = u32::try_from((head + 1) % MAX_RUNTIME_QUEUE_FRAMES).expect("queue head fits u32");
    queue.len -= 1;
    Ok(())
}

pub(crate) fn record_relay_failure() -> Result<(), RuntimeQueueError> {
    require_preloaded()?;
    let mut queue = RUNTIME_QUEUE.exclusive();
    queue.relay_failures = checked_increment(queue.relay_failures, "relay_failures")?;
    Ok(())
}

pub(crate) fn status(capacity: usize) -> RuntimeQueueStatus {
    if !is_preloaded() {
        return status::empty(capacity);
    }
    let queue = RUNTIME_QUEUE.share();
    status::from_state(capacity, &queue)
}

fn require_preloaded() -> Result<(), RuntimeQueueError> {
    is_preloaded()
        .then_some(())
        .ok_or(RuntimeQueueError::NotPreloaded)
}

fn checked_increment(value: u64, field: &'static str) -> Result<u64, RuntimeQueueError> {
    value
        .checked_add(1)
        .ok_or(RuntimeQueueError::CounterOverflow(field))
}

pub(crate) fn format_lsn(lsn: u64) -> String {
    status::format_lsn(lsn)
}
