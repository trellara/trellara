use super::*;

#[test]
fn worker_supervision_drains_only_when_every_runtime_gate_is_ready() {
    let ready_with_frames = worker_state(true, true, true, true);

    let decision = native_worker_supervision_decision(&ready_with_frames).expect("worker decision");

    assert_eq!(decision.contract, WORKER_SUPERVISION_CONTRACT);
    assert!(decision.should_drain);
    assert!(!decision.should_sleep);
}

#[test]
fn worker_supervision_sleeps_when_ready_but_queue_is_empty() {
    let ready_without_frames = worker_state(true, true, true, false);

    let decision =
        native_worker_supervision_decision(&ready_without_frames).expect("worker decision");

    assert!(!decision.should_drain);
    assert!(decision.should_sleep);
}

#[test]
fn worker_supervision_fails_closed_before_hooks_memory_or_relay() {
    assert_eq!(
        native_worker_supervision_decision(&worker_state(false, true, true, true)),
        Err(NativeWorkerSupervisionError::HooksNotReady)
    );
    assert_eq!(
        native_worker_supervision_decision(&worker_state(true, false, true, true)),
        Err(NativeWorkerSupervisionError::SharedMemoryNotReady)
    );
    assert_eq!(
        native_worker_supervision_decision(&worker_state(true, true, false, true)),
        Err(NativeWorkerSupervisionError::RelayNotConfigured)
    );
}

#[test]
fn worker_supervision_fails_closed_when_relay_auth_denies_handoff() {
    let mut state = worker_state(true, true, true, true);
    state
        .relay_auth_plan
        .as_mut()
        .expect("relay auth plan")
        .handoff_allowed = false;

    assert_eq!(
        native_worker_supervision_decision(&state),
        Err(NativeWorkerSupervisionError::RelayNotConfigured)
    );
}

fn worker_state(
    hooks_ready: bool,
    memory_ready: bool,
    relay_configured: bool,
    queue_has_frames: bool,
) -> NativeWorkerSupervisionState {
    let config = native_handoff_queue_config(64, 1024, 16 * 1024).expect("queue config");
    let plan = native_shared_memory_plan(&config).expect("shared memory plan");
    let shared_memory_gate = native_shared_memory_lifecycle_gate(
        &plan,
        if memory_ready {
            NativeSharedMemoryPhase::CaptureStart
        } else {
            NativeSharedMemoryPhase::BackendAttach
        },
        true,
        memory_ready,
    )
    .expect("shared memory lifecycle gate");

    NativeWorkerSupervisionState {
        hook_plan: native_hook_registration_plan(hooks_ready, hooks_ready, hooks_ready),
        shared_memory_gate,
        relay_auth_plan: relay_configured.then(|| {
            native_relay_auth_plan(
                "trellara-relay@cluster-a",
                "pg_parameter:trellara.relay_secret",
                "/var/run/postgresql/trellara-relay.sock",
            )
            .expect("relay auth plan")
        }),
        queue_has_frames,
    }
}
