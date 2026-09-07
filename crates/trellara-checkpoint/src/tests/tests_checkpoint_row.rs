use super::*;

#[test]
fn checkpoint_from_parts_accepts_valid_persisted_row() {
    let checkpoint = checkpoint_row::checkpoint_from_parts(
        "source".to_string(),
        "sales".to_string(),
        "0/16B9000".to_string(),
        "0/16B8000".to_string(),
        "0/16B7000".to_string(),
    )
    .expect("valid checkpoint row");

    assert_eq!(checkpoint.source_id, "source");
    assert_eq!(checkpoint.dataset_id, "sales");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B9000");
}

#[test]
fn checkpoint_from_parts_rejects_corrupted_persisted_row() {
    let error = checkpoint_row::checkpoint_from_parts(
        "source".to_string(),
        "sales".to_string(),
        "0/16B9000".to_string(),
        "0/16B7000".to_string(),
        "0/16B8000".to_string(),
    )
    .expect_err("corrupted checkpoint row");

    assert!(error.to_string().contains("must not be ahead"));
}
