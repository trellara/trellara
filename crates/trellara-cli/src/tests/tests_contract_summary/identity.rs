use super::*;

#[test]
fn contract_test_surfaces_identity_default_primary_key_as_advisory() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let table = trellara_pg_capture::TablePreflight {
            replica_identity: Some(trellara_protocol::ReplicaIdentity::Default),
            contract_notes: vec![
                "replica identity DEFAULT is accepted because primary key (id) can identify UPDATE/DELETE rows; REPLICA IDENTITY FULL is not required for this table"
                    .to_string(),
            ],
            ..preflight_table(42)
        };
    let summary = ContractTestSummary::from_preflight(&config, vec![table]);

    assert!(summary.passed);
    let identity_note = summary
        .checks
        .iter()
        .find(|check| check.name == "cdc_identity:public.sales")
        .expect("cdc identity check");
    assert_eq!(identity_note.severity, ContractSeverity::Warning);
    assert!(identity_note.passed);
    assert!(identity_note
        .message
        .contains("safe with primary key apply"));
    assert!(identity_note
        .message
        .contains("REPLICA IDENTITY FULL is not required"));
}

#[test]
fn contract_test_surfaces_toast_patching_required_for_default_identity() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let table = trellara_pg_capture::TablePreflight {
            replica_identity: Some(trellara_protocol::ReplicaIdentity::Default),
            columns: vec![
                source_column("id", "text"),
                source_column("profile", "jsonb"),
                source_column("description", "character varying"),
            ],
            contract_notes: vec![
                "replica identity DEFAULT may omit unchanged TOAST values for profile, description; downstream apply must treat absent non-key columns as unchanged"
                    .to_string(),
            ],
            ..preflight_table(42)
        };
    let summary = ContractTestSummary::from_preflight(&config, vec![table]);

    assert!(summary.passed);
    let toast = summary
        .checks
        .iter()
        .find(|check| check.name == "toast_patching:public.sales")
        .expect("toast patching check");
    assert!(toast.passed);
    assert_eq!(toast.severity, ContractSeverity::Warning);
    assert!(toast.message.contains("TOAST patching required"));
    assert!(toast.message.contains("preserve absent non-key columns"));
}

#[test]
fn contract_test_reports_preflight_issues() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let table = trellara_pg_capture::TablePreflight {
        issues: vec!["replica identity is unsafe".to_string()],
        ..preflight_table(42)
    };
    let summary = ContractTestSummary::from_preflight(&config, vec![table]);

    assert!(!summary.passed);
    assert_eq!(summary.issue_count, 1);
    assert_eq!(summary.checks[0].severity, ContractSeverity::Error);
    assert!(summary.checks[0]
        .message
        .contains("replica identity is unsafe"));
}

#[test]
fn contract_test_surfaces_unsafe_until_identity_configured() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let table = trellara_pg_capture::TablePreflight {
            replica_identity: Some(trellara_protocol::ReplicaIdentity::Default),
            primary_key_columns: Vec::new(),
            update_delete_safe: false,
            issues: vec![
                "UPDATE/DELETE capture is unsafe without replica identity FULL, identity index, or primary key"
                    .to_string(),
            ],
            ..preflight_table(42)
        };
    let summary = ContractTestSummary::from_preflight(&config, vec![table]);

    assert!(!summary.passed);
    let identity = summary
        .checks
        .iter()
        .find(|check| check.name == "cdc_identity:public.sales")
        .expect("cdc identity check");
    assert!(!identity.passed);
    assert_eq!(identity.severity, ContractSeverity::Error);
    assert!(identity
        .message
        .contains("unsafe until identity configured"));
    assert!(identity.recommendation.contains("REPLICA IDENTITY FULL"));
}
