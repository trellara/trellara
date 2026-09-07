use super::*;

#[test]
fn data_plane_readiness_separates_contracts_from_runtime_gaps() {
    let readiness = native_data_plane_readiness();

    assert!(!readiness.data_plane_ready);
    assert_eq!(readiness.implemented_components.len(), 9);
    assert_eq!(readiness.pending_components.len(), 4);
    assert!(readiness
        .implemented_components
        .iter()
        .all(|component| component.ready));
    assert!(readiness
        .pending_components
        .iter()
        .all(|component| !component.ready));
}

#[test]
fn data_plane_readiness_names_differentiated_native_boundaries() {
    let readiness = native_data_plane_readiness();
    let implemented: Vec<_> = readiness
        .implemented_components
        .iter()
        .map(|component| component.name)
        .collect();

    assert!(implemented.contains(&"committed_transaction_frame_contract"));
    assert!(implemented.contains(&"bounded_queue_admission"));
    assert!(implemented.contains(&"ordered_worker_drain_contract"));
    assert!(implemented.contains(&"relation_metadata_identity"));
    assert!(implemented.contains(&"source_feedback_decision_contract"));
    assert!(implemented.contains(&"shared_memory_allocation_plan"));
    assert!(implemented.contains(&"native_hook_registration_plan"));
    assert!(implemented.contains(&"background_worker_supervision_contract"));
    assert!(implemented.contains(&"relay_peer_auth_contract"));
}
