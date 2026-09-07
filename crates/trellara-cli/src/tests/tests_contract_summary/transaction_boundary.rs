use super::*;

#[test]
fn contract_test_surfaces_strict_chunk_manifest_boundary() {
    let yaml = STRICT_YAML.replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 1000",
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");
    let summary = ContractTestSummary::from_preflight(&config, vec![preflight_table(42)]);

    assert!(summary.passed);
    let boundary = summary
        .checks
        .iter()
        .find(|check| check.name == "transaction_boundary:strict_chunk_manifest")
        .expect("strict chunk boundary");
    assert!(boundary
        .message
        .contains("manifest and commit marker barrier"));
    assert!(boundary.message.contains("1000 changes"));
}
