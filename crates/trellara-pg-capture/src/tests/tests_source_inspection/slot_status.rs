use crate::source_inspection::slot_status_issues;

#[test]
fn slot_status_reports_missing_slot_issue() {
    assert_eq!(
        slot_status_issues(false, None, "pgoutput", None, None, None, None),
        vec!["replication slot does not exist".to_string()]
    );
}

#[test]
fn slot_status_reports_plugin_mismatch() {
    assert_eq!(
        slot_status_issues(
            true,
            Some("test_decoding"),
            "pgoutput",
            Some(0),
            None,
            None,
            None
        ),
        vec!["replication slot plugin is test_decoding, expected pgoutput".to_string()]
    );
}

#[test]
fn slot_status_reports_unknown_plugin_for_existing_slot() {
    assert_eq!(
        slot_status_issues(true, None, "pgoutput", Some(0), None, None, None),
        vec!["replication slot plugin is unknown, expected pgoutput".to_string()]
    );
}

#[test]
fn slot_status_accepts_matching_plugin_and_retention() {
    assert!(slot_status_issues(
        true,
        Some("test_decoding"),
        "test_decoding",
        Some(0),
        Some("reserved"),
        Some(1_000_000),
        None
    )
    .is_empty());
}

#[test]
fn slot_status_reports_lost_wal() {
    assert_eq!(
        slot_status_issues(
            true,
            Some("test_decoding"),
            "test_decoding",
            Some(0),
            Some("lost"),
            None,
            None
        ),
        vec![
            "replication slot WAL is lost; recreate the slot and reseed affected targets"
                .to_string()
        ]
    );
}

#[test]
fn slot_status_reports_invalidation_reason() {
    assert_eq!(
        slot_status_issues(
            true,
            Some("test_decoding"),
            "test_decoding",
            Some(0),
            Some("reserved"),
            Some(1_000_000),
            Some("wal_removed")
        ),
        vec![
            "replication slot invalidated: wal_removed; recreate the slot and reseed affected targets"
                .to_string()
        ]
    );
}

#[test]
fn slot_status_reports_exhausted_safe_wal_size() {
    assert_eq!(
        slot_status_issues(
            true,
            Some("test_decoding"),
            "test_decoding",
            Some(0),
            Some("reserved"),
            Some(0),
            None
        ),
        vec![
            "replication slot safe WAL size is exhausted; drain or reseed before continuing"
                .to_string()
        ]
    );
}
