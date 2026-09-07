use std::time::Duration;

use pgrx::bgworkers::{BackgroundWorkerBuilder, BgWorkerStartTime};
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct NativeRuntimeStatus {
    pub data_plane_ready: bool,
    pub preloaded: bool,
    pub enabled: bool,
    pub database_name: String,
    pub slot_name: String,
    pub source_id: String,
    pub dataset_id: String,
    pub relay_socket_path: String,
    pub relay_secret_configured: bool,
    pub queue: crate::pg_shared_queue::RuntimeQueueStatus,
}

pub(crate) fn initialize() {
    crate::pg_guc::register();
    if unsafe { pgrx::pg_sys::process_shared_preload_libraries_in_progress } {
        crate::pg_shared_queue::initialize();
        BackgroundWorkerBuilder::new("Trellara native relay")
            .set_type("trellara_native_relay")
            .set_library("trellara_pg_extension")
            .set_function("trellara_native_worker_main")
            .enable_spi_access()
            .set_start_time(BgWorkerStartTime::RecoveryFinished)
            .set_restart_time(Some(Duration::from_secs(1)))
            .load();
    }
}

pub(crate) fn status() -> NativeRuntimeStatus {
    let queue = crate::pg_shared_queue::status(crate::pg_guc::queue_capacity());
    let enabled = crate::pg_guc::enabled();
    let database_name = crate::pg_guc::database_name();
    let slot_name = crate::pg_guc::slot_name();
    let source_id = crate::pg_guc::source_id();
    let dataset_id = crate::pg_guc::dataset_id();
    let relay_socket_path = crate::pg_guc::relay_socket_path();
    let relay_secret_configured = !crate::pg_guc::relay_secret().is_empty();
    let data_plane_ready = queue.preloaded
        && enabled
        && queue.worker_starts > 0
        && !database_name.is_empty()
        && !slot_name.is_empty()
        && !source_id.is_empty()
        && !dataset_id.is_empty()
        && relay_socket_path.starts_with('/')
        && relay_secret_configured;
    NativeRuntimeStatus {
        data_plane_ready,
        preloaded: queue.preloaded,
        enabled,
        database_name,
        slot_name,
        source_id,
        dataset_id,
        relay_socket_path,
        relay_secret_configured,
        queue,
    }
}

pub(crate) fn data_plane_ready() -> bool {
    status().data_plane_ready
}
