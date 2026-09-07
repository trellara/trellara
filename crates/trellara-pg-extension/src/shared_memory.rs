use serde::Serialize;

use crate::{NativeHandoffQueueConfig, MAX_HANDOFF_QUEUE_FRAMES};

pub const SHARED_MEMORY_LIFECYCLE_CONTRACT: &str =
    "allocated_at_postmaster_start_and_bounded_before_capture_enabled";
const QUEUE_HEADER_BYTES: usize = 4096;
const QUEUE_FRAME_DESCRIPTOR_BYTES: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeSharedMemoryPlan {
    pub contract: &'static str,
    pub capacity_frames: usize,
    pub max_frame_payload_bytes: usize,
    pub max_buffered_payload_bytes: usize,
    pub frame_descriptor_bytes: usize,
    pub header_bytes: usize,
    pub required_bytes: usize,
    pub shared_preload_required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum NativeSharedMemoryPhase {
    PostmasterStart,
    BackendAttach,
    CaptureStart,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeSharedMemoryLifecycleGate {
    pub contract: &'static str,
    pub phase: NativeSharedMemoryPhase,
    pub required_bytes: usize,
    pub capture_allowed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeSharedMemoryPlanError {
    InvalidFrameCapacity,
    InvalidFramePayloadLimit,
    DescriptorBytesOverflow,
    RequiredBytesOverflow,
    RegistrationAfterPostmasterStart,
    CaptureBeforeBackendAttach,
}

pub fn native_shared_memory_plan(
    config: &NativeHandoffQueueConfig,
) -> Result<NativeSharedMemoryPlan, NativeSharedMemoryPlanError> {
    if config.capacity_frames == 0 || config.capacity_frames > MAX_HANDOFF_QUEUE_FRAMES {
        return Err(NativeSharedMemoryPlanError::InvalidFrameCapacity);
    }
    if config.max_frame_payload_bytes == 0
        || config.max_frame_payload_bytes > config.max_buffered_payload_bytes
    {
        return Err(NativeSharedMemoryPlanError::InvalidFramePayloadLimit);
    }

    let frame_descriptor_bytes = config
        .capacity_frames
        .checked_mul(QUEUE_FRAME_DESCRIPTOR_BYTES)
        .ok_or(NativeSharedMemoryPlanError::DescriptorBytesOverflow)?;
    let required_bytes = QUEUE_HEADER_BYTES
        .checked_add(frame_descriptor_bytes)
        .and_then(|bytes| bytes.checked_add(config.max_buffered_payload_bytes))
        .ok_or(NativeSharedMemoryPlanError::RequiredBytesOverflow)?;

    Ok(NativeSharedMemoryPlan {
        contract: SHARED_MEMORY_LIFECYCLE_CONTRACT,
        capacity_frames: config.capacity_frames,
        max_frame_payload_bytes: config.max_frame_payload_bytes,
        max_buffered_payload_bytes: config.max_buffered_payload_bytes,
        frame_descriptor_bytes,
        header_bytes: QUEUE_HEADER_BYTES,
        required_bytes,
        shared_preload_required: true,
    })
}

pub fn native_shared_memory_lifecycle_gate(
    plan: &NativeSharedMemoryPlan,
    phase: NativeSharedMemoryPhase,
    registered_at_postmaster_start: bool,
    backend_attached: bool,
) -> Result<NativeSharedMemoryLifecycleGate, NativeSharedMemoryPlanError> {
    if !registered_at_postmaster_start {
        return Err(NativeSharedMemoryPlanError::RegistrationAfterPostmasterStart);
    }
    if phase == NativeSharedMemoryPhase::CaptureStart && !backend_attached {
        return Err(NativeSharedMemoryPlanError::CaptureBeforeBackendAttach);
    }

    Ok(NativeSharedMemoryLifecycleGate {
        contract: SHARED_MEMORY_LIFECYCLE_CONTRACT,
        phase,
        required_bytes: plan.required_bytes,
        capture_allowed: phase == NativeSharedMemoryPhase::CaptureStart,
    })
}
