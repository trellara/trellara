use super::*;

#[test]
fn protocol_property_tests_accept_split_property_modules() {
    let root = temp_root("protocol-properties-split-ready");
    let src = root
        .join("crates")
        .join("trellara-protocol")
        .join("src")
        .join("tests")
        .join("tests_properties");
    fs::create_dir_all(&src).expect("create protocol temp dir");
    fs::write(
        src.join("mod.rs"),
        "mod envelope; mod idempotency_lsn; mod strict_chunk_manifest; mod partitioned_manifest;",
    )
    .expect("write property mod");
    fs::write(
        src.join("envelope.rs"),
        "proptest! { envelope_roundtrip_property_preserves_checksum }",
    )
    .expect("write envelope properties");
    fs::write(
        src.join("idempotency_lsn.rs"),
        "proptest! {
            idempotency_key_property_is_deterministic,
            lsn_ordering_property_matches_numeric_order
        }",
    )
    .expect("write idempotency properties");
    fs::write(
        src.join("strict_chunk_manifest.rs"),
        "proptest! {
            strict_chunk_manifest_property_reconstructs_source_order,
            strict_chunk_manifest_property_rejects_missing_chunk,
            strict_chunk_manifest_property_rejects_duplicate_chunk
        }",
    )
    .expect("write strict chunk properties");
    fs::write(
        src.join("partitioned_manifest.rs"),
        "proptest! {
            partitioned_manifest_property_reconstructs_source_order,
            partitioned_manifest_property_rejects_missing_partition_chunk,
            partitioned_manifest_property_rejects_duplicate_partition_chunk
        }",
    )
    .expect("write partitioned properties");

    assert!(protocol_property_tests_are_current(&root));

    fs::remove_dir_all(root).expect("remove protocol temp dir");
}

#[test]
fn protocol_property_tests_reject_generic_proptest_block() {
    let root = temp_root("protocol-properties-stale");
    let src = root.join("crates").join("trellara-protocol").join("src");
    fs::create_dir_all(&src).expect("create protocol temp dir");
    fs::write(
        src.join("tests.rs"),
        "proptest! { generic_protocol_property }",
    )
    .expect("write protocol source");

    assert!(!protocol_property_tests_are_current(&root));

    fs::remove_dir_all(root).expect("remove protocol temp dir");
}

#[test]
fn protocol_property_tests_reject_single_file_property_bundle() {
    let root = temp_root("protocol-properties-single-file-stale");
    let src = root
        .join("crates")
        .join("trellara-protocol")
        .join("src")
        .join("tests");
    fs::create_dir_all(&src).expect("create protocol temp dir");
    fs::write(
        src.join("tests_properties.rs"),
        "proptest! {
                envelope_roundtrip_property_preserves_checksum,
                idempotency_key_property_is_deterministic,
                lsn_ordering_property_matches_numeric_order,
                strict_chunk_manifest_property_reconstructs_source_order,
                strict_chunk_manifest_property_rejects_missing_chunk,
                strict_chunk_manifest_property_rejects_duplicate_chunk,
                partitioned_manifest_property_reconstructs_source_order,
                partitioned_manifest_property_rejects_missing_partition_chunk,
                partitioned_manifest_property_rejects_duplicate_partition_chunk
            }",
    )
    .expect("write protocol source");

    assert!(!protocol_property_tests_are_current(&root));

    fs::remove_dir_all(root).expect("remove protocol temp dir");
}

#[test]
fn protocol_property_tests_reject_missing_duplicate_rejection_properties() {
    let root = temp_root("protocol-properties-missing-duplicates");
    let src = root
        .join("crates")
        .join("trellara-protocol")
        .join("src")
        .join("tests")
        .join("tests_properties");
    fs::create_dir_all(&src).expect("create protocol temp dir");
    fs::write(
        src.join("mod.rs"),
        "mod envelope; mod idempotency_lsn; mod strict_chunk_manifest; mod partitioned_manifest;",
    )
    .expect("write property mod");
    fs::write(
        src.join("envelope.rs"),
        "proptest! { envelope_roundtrip_property_preserves_checksum }",
    )
    .expect("write envelope properties");
    fs::write(
        src.join("idempotency_lsn.rs"),
        "proptest! {
            idempotency_key_property_is_deterministic,
            lsn_ordering_property_matches_numeric_order
        }",
    )
    .expect("write idempotency properties");
    fs::write(
        src.join("strict_chunk_manifest.rs"),
        "proptest! {
            strict_chunk_manifest_property_reconstructs_source_order,
            strict_chunk_manifest_property_rejects_missing_chunk
        }",
    )
    .expect("write strict chunk properties");
    fs::write(
        src.join("partitioned_manifest.rs"),
        "proptest! {
            partitioned_manifest_property_reconstructs_source_order,
            partitioned_manifest_property_rejects_missing_partition_chunk
        }",
    )
    .expect("write partitioned properties");

    assert!(!protocol_property_tests_are_current(&root));

    fs::remove_dir_all(root).expect("remove protocol temp dir");
}
