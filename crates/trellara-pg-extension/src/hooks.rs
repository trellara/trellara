use serde::Serialize;

pub const HOOK_REGISTRATION_CONTRACT: &str =
    "postmaster_registers_shared_memory_before_workers_or_capture_callbacks";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeHookRegistrationPlan {
    pub contract: &'static str,
    pub steps: Vec<NativeHookRegistrationStep>,
    pub data_plane_hooks_ready: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeHookRegistrationStep {
    pub name: &'static str,
    pub required_phase: &'static str,
    pub wired: bool,
}

pub fn native_hook_registration_plan(
    shared_memory_registered: bool,
    background_worker_registered: bool,
    logical_decoding_registered: bool,
) -> NativeHookRegistrationPlan {
    let steps = vec![
        step(
            "request_addin_shmem_space",
            "postmaster_start",
            shared_memory_registered,
        ),
        step(
            "register_background_worker",
            "postmaster_start_after_shared_memory",
            background_worker_registered,
        ),
        step(
            "logical_decoding_callbacks",
            "replication_slot_start_after_backend_attach",
            logical_decoding_registered,
        ),
    ];
    NativeHookRegistrationPlan {
        contract: HOOK_REGISTRATION_CONTRACT,
        data_plane_hooks_ready: steps.iter().all(|step| step.wired),
        steps,
    }
}

fn step(
    name: &'static str,
    required_phase: &'static str,
    wired: bool,
) -> NativeHookRegistrationStep {
    NativeHookRegistrationStep {
        name,
        required_phase,
        wired,
    }
}
