#[pgrx::pg_schema]
mod trellara {
    use pgrx::JsonB;

    #[pgrx::pg_extern]
    fn extension_version() -> &'static str {
        crate::EXTENSION_VERSION
    }

    #[pgrx::pg_extern]
    fn protocol_version() -> i32 {
        i32::try_from(trellara_protocol::PROTOCOL_VERSION)
            .expect("Trellara protocol version fits a PostgreSQL integer")
    }

    #[pgrx::pg_extern]
    fn extension_status() -> JsonB {
        let postgres_major = crate::compiled_postgres_major()
            .expect("a PostgreSQL feature is required for the extension SQL API");
        JsonB(
            serde_json::to_value(crate::NativeExtensionStatus::for_postgres_major(
                postgres_major,
            ))
            .expect("native extension status is JSON serializable"),
        )
    }

    #[pgrx::pg_extern]
    fn data_plane_readiness() -> JsonB {
        JsonB(
            serde_json::to_value(crate::native_data_plane_readiness())
                .expect("native data plane readiness is JSON serializable"),
        )
    }

    #[pgrx::pg_extern]
    fn runtime_status() -> JsonB {
        JsonB(
            serde_json::to_value(crate::pg_runtime::status())
                .expect("native runtime status is JSON serializable"),
        )
    }

    #[pgrx::pg_extern]
    fn shared_memory_plan(
        capacity_frames: i64,
        max_frame_payload_bytes: i64,
        max_buffered_payload_bytes: i64,
    ) -> JsonB {
        match shared_memory_plan_json(
            capacity_frames,
            max_frame_payload_bytes,
            max_buffered_payload_bytes,
        ) {
            Ok(plan) => JsonB(plan),
            Err(error) => pgrx::error!("{error}"),
        }
    }

    #[pgrx::pg_extern]
    fn capture_contract(source_id: &str, dataset_id: &str) -> JsonB {
        let postgres_major = crate::compiled_postgres_major()
            .expect("a PostgreSQL feature is required for the extension SQL API");
        match crate::native_capture_contract(source_id, dataset_id, postgres_major) {
            Ok(contract) => JsonB(
                serde_json::to_value(contract)
                    .expect("native capture contract is JSON serializable"),
            ),
            Err(error) => pgrx::error!("{error}"),
        }
    }

    fn shared_memory_plan_json(
        capacity_frames: i64,
        max_frame_payload_bytes: i64,
        max_buffered_payload_bytes: i64,
    ) -> Result<serde_json::Value, String> {
        let config = crate::native_handoff_queue_config(
            positive_usize(capacity_frames, "capacity_frames")?,
            positive_usize(max_frame_payload_bytes, "max_frame_payload_bytes")?,
            positive_usize(max_buffered_payload_bytes, "max_buffered_payload_bytes")?,
        )
        .map_err(|error| format!("{error:?}"))?;
        crate::native_shared_memory_plan(&config)
            .map_err(|error| format!("{error:?}"))
            .and_then(|plan| {
                serde_json::to_value(plan)
                    .map_err(|error| format!("native shared memory plan is not JSON: {error}"))
            })
    }

    fn positive_usize(value: i64, field: &str) -> Result<usize, String> {
        usize::try_from(value).map_err(|_| format!("{field} must be positive"))
    }
}
