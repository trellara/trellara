use super::*;

#[test]
fn partition_checkpoint_from_parts_accepts_valid_persisted_row() {
    let checkpoint = partition_checkpoint_row::partition_checkpoint_from_parts(
        "source".to_string(),
        "sales".to_string(),
        2,
        "0/16B9000".to_string(),
        "0/16B8000".to_string(),
    )
    .expect("valid partition checkpoint row");

    assert_eq!(checkpoint.partition_id, 2);
    assert_eq!(checkpoint.last_applied_lsn, "0/16B8000");
}

#[test]
fn partition_checkpoint_from_parts_rejects_corrupted_persisted_row() {
    let error = partition_checkpoint_row::partition_checkpoint_from_parts(
        "source".to_string(),
        "sales".to_string(),
        2,
        "0/16B8000".to_string(),
        "0/16B9000".to_string(),
    )
    .expect_err("corrupted partition checkpoint row");

    assert!(error.to_string().contains("applied LSN"));
    assert!(error.to_string().contains("ahead of durable LSN"));
}
