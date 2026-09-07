use serde::Serialize;

use crate::{NativeHookRegistrationPlan, NativeRelayAuthPlan, NativeSharedMemoryLifecycleGate};

pub const WORKER_SUPERVISION_CONTRACT: &str =
    "background_worker_drains_only_when_hooks_memory_and_relay_are_ready";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeWorkerSupervisionState {
    pub hook_plan: NativeHookRegistrationPlan,
    pub shared_memory_gate: NativeSharedMemoryLifecycleGate,
    pub relay_auth_plan: Option<NativeRelayAuthPlan>,
    pub queue_has_frames: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeWorkerSupervisionDecision {
    pub contract: &'static str,
    pub should_drain: bool,
    pub should_sleep: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWorkerSupervisionError {
    HooksNotReady,
    SharedMemoryNotReady,
    RelayNotConfigured,
}

pub fn native_worker_supervision_decision(
    state: &NativeWorkerSupervisionState,
) -> Result<NativeWorkerSupervisionDecision, NativeWorkerSupervisionError> {
    if !state.hook_plan.data_plane_hooks_ready {
        return Err(NativeWorkerSupervisionError::HooksNotReady);
    }
    if !state.shared_memory_gate.capture_allowed {
        return Err(NativeWorkerSupervisionError::SharedMemoryNotReady);
    }
    if !state
        .relay_auth_plan
        .as_ref()
        .is_some_and(|plan| plan.handoff_allowed)
    {
        return Err(NativeWorkerSupervisionError::RelayNotConfigured);
    }

    Ok(NativeWorkerSupervisionDecision {
        contract: WORKER_SUPERVISION_CONTRACT,
        should_drain: state.queue_has_frames,
        should_sleep: !state.queue_has_frames,
    })
}
