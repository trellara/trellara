use super::*;

#[test]
fn epoch_id_is_stable_across_source_ordering() {
    let left = deterministic_epoch_id(
        "retail-sales",
        ["store-001", "store-002"],
        &LakeStragglerPolicy::PublishWithGaps { grace_ms: 30_000 },
        &[
            LakeEpochSourceWindow::new("store-002", Some("0/16B6D00"), Some("0/16B6D20")),
            LakeEpochSourceWindow::new("store-001", Some("0/16B6C00"), Some("0/16B6C50")),
        ],
    )
    .expect("epoch id");
    let right = deterministic_epoch_id(
        "retail-sales",
        ["store-002", "store-001"],
        &LakeStragglerPolicy::PublishWithGaps { grace_ms: 30_000 },
        &[
            LakeEpochSourceWindow::new("store-001", Some("0/16B6C00"), Some("0/16B6C50")),
            LakeEpochSourceWindow::new("store-002", Some("0/16B6D00"), Some("0/16B6D20")),
        ],
    )
    .expect("epoch id");

    assert_eq!(left, right);
    assert!(left.starts_with("epoch-retail-sales-"));
}

#[test]
fn epoch_id_changes_when_policy_or_window_changes() {
    let base = deterministic_epoch_id(
        "retail",
        ["store-001", "store-002"],
        &LakeStragglerPolicy::WaitAllRequired,
        &[LakeEpochSourceWindow::new(
            "store-001",
            Some("0/16B6C00"),
            Some("0/16B6C50"),
        )],
    )
    .expect("base id");
    let different_policy = deterministic_epoch_id(
        "retail",
        ["store-001", "store-002"],
        &LakeStragglerPolicy::PublishWithGaps { grace_ms: 0 },
        &[LakeEpochSourceWindow::new(
            "store-001",
            Some("0/16B6C00"),
            Some("0/16B6C50"),
        )],
    )
    .expect("policy id");
    let different_window = deterministic_epoch_id(
        "retail",
        ["store-001", "store-002"],
        &LakeStragglerPolicy::WaitAllRequired,
        &[LakeEpochSourceWindow::new(
            "store-001",
            Some("0/16B6C00"),
            Some("0/16B6C70"),
        )],
    )
    .expect("window id");

    assert_ne!(base, different_policy);
    assert_ne!(base, different_window);
}

#[test]
fn duplicate_source_window_fails_closed() {
    let error = deterministic_epoch_id(
        "retail",
        ["store-001"],
        &LakeStragglerPolicy::WaitAllRequired,
        &[
            LakeEpochSourceWindow::new("store-001", Some("0/16B6C00"), Some("0/16B6C50")),
            LakeEpochSourceWindow::new("store-001", Some("0/16B6C50"), Some("0/16B6C70")),
        ],
    )
    .expect_err("duplicate source");

    assert!(matches!(
        error,
        LakeError::DuplicateEpochSourceWindow { source_id } if source_id == "store-001"
    ));
}
