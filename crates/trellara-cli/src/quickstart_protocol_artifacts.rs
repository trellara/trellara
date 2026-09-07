use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn protocol_property_tests_are_current(repository_root: &Path) -> bool {
    let test_dir = repository_root
        .join("crates")
        .join("trellara-protocol")
        .join("src")
        .join("tests")
        .join("tests_properties");

    required_property_modules(&test_dir).is_some_and(|modules| {
        module_has_property(
            &modules.envelope,
            "envelope_roundtrip_property_preserves_checksum",
        ) && module_has_property(
            &modules.idempotency_lsn,
            "idempotency_key_property_is_deterministic",
        ) && module_has_property(
            &modules.idempotency_lsn,
            "lsn_ordering_property_matches_numeric_order",
        ) && module_has_property(
            &modules.strict_chunk_manifest,
            "strict_chunk_manifest_property_reconstructs_source_order",
        ) && module_has_property(
            &modules.strict_chunk_manifest,
            "strict_chunk_manifest_property_rejects_missing_chunk",
        ) && module_has_property(
            &modules.strict_chunk_manifest,
            "strict_chunk_manifest_property_rejects_duplicate_chunk",
        ) && module_has_property(
            &modules.partitioned_manifest,
            "partitioned_manifest_property_reconstructs_source_order",
        ) && module_has_property(
            &modules.partitioned_manifest,
            "partitioned_manifest_property_rejects_missing_partition_chunk",
        ) && module_has_property(
            &modules.partitioned_manifest,
            "partitioned_manifest_property_rejects_duplicate_partition_chunk",
        )
    })
}

struct ProtocolPropertyModules {
    envelope: String,
    idempotency_lsn: String,
    strict_chunk_manifest: String,
    partitioned_manifest: String,
}

fn required_property_modules(test_dir: &Path) -> Option<ProtocolPropertyModules> {
    if !test_dir.join("mod.rs").is_file() {
        return None;
    }

    Some(ProtocolPropertyModules {
        envelope: read_module(test_dir, "envelope.rs")?,
        idempotency_lsn: read_module(test_dir, "idempotency_lsn.rs")?,
        strict_chunk_manifest: read_module(test_dir, "strict_chunk_manifest.rs")?,
        partitioned_manifest: read_module(test_dir, "partitioned_manifest.rs")?,
    })
}

fn read_module(test_dir: &Path, file: &str) -> Option<String> {
    fs::read_to_string(PathBuf::from(test_dir).join(file)).ok()
}

fn module_has_property(module: &str, property_name: &str) -> bool {
    module.contains("proptest!") && module.contains(property_name)
}
