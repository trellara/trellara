use serde::Serialize;

use crate::{
    HANDOFF_CONTRACT, HOOK_REGISTRATION_CONTRACT, RELAY_AUTH_CONTRACT, RELAY_HANDOFF_CONTRACT,
    REQUIRED_REPLICA_IDENTITY, SHARED_MEMORY_LIFECYCLE_CONTRACT, SOURCE_ACKNOWLEDGEMENT_CONTRACT,
    WORKER_SUPERVISION_CONTRACT,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeDataPlaneReadiness {
    pub data_plane_ready: bool,
    pub implemented_components: Vec<NativeDataPlaneReadinessComponent>,
    pub pending_components: Vec<NativeDataPlaneReadinessComponent>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeDataPlaneReadinessComponent {
    pub name: &'static str,
    pub contract: &'static str,
    pub ready: bool,
}

pub fn native_data_plane_readiness() -> NativeDataPlaneReadiness {
    let mut implemented_components = vec![
        component(
            "committed_transaction_frame_contract",
            "validates exact source transaction boundary before native handoff",
            true,
        ),
        component("bounded_queue_admission", HANDOFF_CONTRACT, true),
        component(
            "ordered_worker_drain_contract",
            RELAY_HANDOFF_CONTRACT,
            true,
        ),
        component(
            "relation_metadata_identity",
            REQUIRED_REPLICA_IDENTITY,
            true,
        ),
        component(
            "source_feedback_decision_contract",
            SOURCE_ACKNOWLEDGEMENT_CONTRACT,
            true,
        ),
        component(
            "shared_memory_allocation_plan",
            SHARED_MEMORY_LIFECYCLE_CONTRACT,
            true,
        ),
        component(
            "native_hook_registration_plan",
            HOOK_REGISTRATION_CONTRACT,
            true,
        ),
        component(
            "background_worker_supervision_contract",
            WORKER_SUPERVISION_CONTRACT,
            true,
        ),
        component("relay_peer_auth_contract", RELAY_AUTH_CONTRACT, true),
    ];
    let runtime_components = [
        component(
            "pgrx_shared_memory_hook_wiring",
            "preload-time PostgreSQL LWLock-protected bounded queue",
            crate::runtime_data_plane_ready(),
        ),
        component(
            "logical_decoding_callbacks",
            "relation DML truncate message origin-filter commit and streaming callbacks",
            crate::runtime_data_plane_ready(),
        ),
        component(
            "background_worker_runtime",
            "supervised slot peek and local Unix socket relay handoff",
            crate::runtime_data_plane_ready(),
        ),
        component(
            "source_feedback_integration",
            "pg_replication_slot_advance only after authenticated durable relay proof",
            crate::runtime_data_plane_ready(),
        ),
    ];
    let mut pending_components = Vec::new();
    for component in runtime_components {
        if component.ready {
            implemented_components.push(component);
        } else {
            pending_components.push(component);
        }
    }

    NativeDataPlaneReadiness {
        data_plane_ready: pending_components.is_empty(),
        implemented_components,
        pending_components,
    }
}

fn component(
    name: &'static str,
    contract: &'static str,
    ready: bool,
) -> NativeDataPlaneReadinessComponent {
    NativeDataPlaneReadinessComponent {
        name,
        contract,
        ready,
    }
}
