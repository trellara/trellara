use super::*;

#[test]
fn shared_memory_plan_calculates_bounded_queue_allocation() {
    let config = native_handoff_queue_config(64, 1024, 16 * 1024).expect("queue config");

    let plan = native_shared_memory_plan(&config).expect("shared memory plan");

    assert_eq!(plan.contract, SHARED_MEMORY_LIFECYCLE_CONTRACT);
    assert_eq!(plan.capacity_frames, 64);
    assert_eq!(plan.max_frame_payload_bytes, 1024);
    assert_eq!(plan.max_buffered_payload_bytes, 16 * 1024);
    assert_eq!(plan.header_bytes, 4096);
    assert_eq!(plan.frame_descriptor_bytes, 64 * 64);
    assert_eq!(plan.required_bytes, 4096 + 4096 + 16 * 1024);
    assert!(plan.shared_preload_required);
}

#[test]
fn shared_memory_plan_rejects_invalid_unbounded_layouts() {
    let mut config = native_handoff_queue_config(1, 1024, 1024).expect("queue config");

    config.capacity_frames = 0;
    assert_eq!(
        native_shared_memory_plan(&config),
        Err(NativeSharedMemoryPlanError::InvalidFrameCapacity)
    );

    config.capacity_frames = MAX_HANDOFF_QUEUE_FRAMES + 1;
    assert_eq!(
        native_shared_memory_plan(&config),
        Err(NativeSharedMemoryPlanError::InvalidFrameCapacity)
    );

    config.capacity_frames = 1;
    config.max_frame_payload_bytes = 0;
    assert_eq!(
        native_shared_memory_plan(&config),
        Err(NativeSharedMemoryPlanError::InvalidFramePayloadLimit)
    );
}

#[test]
fn shared_memory_lifecycle_allows_capture_only_after_attach() {
    let config = native_handoff_queue_config(64, 1024, 16 * 1024).expect("queue config");
    let plan = native_shared_memory_plan(&config).expect("shared memory plan");

    let attach_gate = native_shared_memory_lifecycle_gate(
        &plan,
        NativeSharedMemoryPhase::BackendAttach,
        true,
        false,
    )
    .expect("backend attach gate");
    assert!(!attach_gate.capture_allowed);

    let capture_gate = native_shared_memory_lifecycle_gate(
        &plan,
        NativeSharedMemoryPhase::CaptureStart,
        true,
        true,
    )
    .expect("capture start gate");
    assert!(capture_gate.capture_allowed);
    assert_eq!(capture_gate.required_bytes, plan.required_bytes);
}

#[test]
fn shared_memory_lifecycle_fails_closed_before_registration_or_attach() {
    let config = native_handoff_queue_config(64, 1024, 16 * 1024).expect("queue config");
    let plan = native_shared_memory_plan(&config).expect("shared memory plan");

    assert_eq!(
        native_shared_memory_lifecycle_gate(
            &plan,
            NativeSharedMemoryPhase::BackendAttach,
            false,
            false,
        ),
        Err(NativeSharedMemoryPlanError::RegistrationAfterPostmasterStart)
    );
    assert_eq!(
        native_shared_memory_lifecycle_gate(
            &plan,
            NativeSharedMemoryPhase::CaptureStart,
            true,
            false,
        ),
        Err(NativeSharedMemoryPlanError::CaptureBeforeBackendAttach)
    );
}
