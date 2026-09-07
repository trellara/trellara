use serde::Serialize;

use crate::{
    COMPILED_POSTGRES_MAJOR, EXTENSION_VERSION, HANDOFF_CONTRACT, SOURCE_ACKNOWLEDGEMENT_CONTRACT,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeExtensionStatus {
    pub extension_name: &'static str,
    pub extension_version: &'static str,
    pub protocol_version: u32,
    pub postgres_major: u16,
    pub postgres_major_supported: bool,
    pub sql_api_ready: bool,
    pub data_plane_ready: bool,
    pub implementation_phase: &'static str,
    pub planned_handoff: &'static str,
    pub source_acknowledgement: &'static str,
    pub shared_preload_required_for_sql_api: bool,
    pub shared_preload_required_for_data_plane: bool,
}

impl NativeExtensionStatus {
    #[must_use]
    pub fn for_postgres_major(postgres_major: u16) -> Self {
        let data_plane_ready = crate::runtime_data_plane_ready();
        Self {
            extension_name: "trellara_pg_extension",
            extension_version: EXTENSION_VERSION,
            protocol_version: trellara_protocol::PROTOCOL_VERSION,
            postgres_major,
            postgres_major_supported: supports_postgres_major(postgres_major),
            sql_api_ready: true,
            data_plane_ready,
            implementation_phase: if data_plane_ready {
                "native_runtime"
            } else {
                "runtime_requires_preload_and_configuration"
            },
            planned_handoff: HANDOFF_CONTRACT,
            source_acknowledgement: SOURCE_ACKNOWLEDGEMENT_CONTRACT,
            shared_preload_required_for_sql_api: false,
            shared_preload_required_for_data_plane: true,
        }
    }
}

#[must_use]
pub const fn supports_postgres_major(postgres_major: u16) -> bool {
    matches!(postgres_major, 15..=18)
}

#[must_use]
pub const fn compiled_postgres_major() -> Option<u16> {
    COMPILED_POSTGRES_MAJOR
}
