use super::*;
use std::fs;

#[test]
fn assembler_removes_spill_file_after_stream_commit() {
    let spill_dir = stream_spill_test_dir("commit-cleanup");
    let mut assembler = TransactionAssembler::with_stream_spill_config(1, Some(spill_dir.clone()));
    let config = assembler_config();

    spill_two_changes(&mut assembler, &config);
    let spill_path = spilled_path(&assembler);
    assert!(spill_path.exists());

    let envelope = assembler
        .apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect("stream commit")
        .expect("streamed envelope");

    assert_eq!(envelope.changes.len(), 2);
    assert!(!spill_path.exists());
    fs::remove_dir_all(spill_dir).expect("remove spill test dir");
}

#[test]
fn assembler_rejects_corrupt_spill_file_on_stream_commit() {
    let spill_dir = stream_spill_test_dir("commit-corrupt-spill");
    let mut assembler = TransactionAssembler::with_stream_spill_config(1, Some(spill_dir.clone()));
    let config = assembler_config();

    spill_two_changes(&mut assembler, &config);
    let spill_path = spilled_path(&assembler);
    fs::write(&spill_path, "{\"total_order\":1}\nnot-json\n").expect("corrupt spill file");

    let error = assembler
        .apply(
            &config,
            LogicalEvent::StreamCommit {
                transaction_id: "42".to_string(),
                commit_lsn: "0/16B6C50".to_string(),
                commit_timestamp_ms: 1_786_420_000_000,
            },
        )
        .expect_err("corrupt spill should fail closed");

    assert!(matches!(
        error,
        CaptureError::StreamSpillCorrupt { line: 1, .. }
    ));
    assert!(error
        .to_string()
        .contains(spill_path.to_string_lossy().as_ref()));
    fs::remove_dir_all(spill_dir).expect("remove spill test dir");
}

fn spill_two_changes(assembler: &mut TransactionAssembler, config: &TransactionAssemblerConfig) {
    record_sales_relation_metadata(assembler, config);
    assembler
        .apply(
            config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start");
    assembler
        .apply(config, streamed_insert_event("42", "sale-1"))
        .expect("first streamed change");
    assembler
        .apply(config, streamed_insert_event("42", "sale-2"))
        .expect("second streamed change");
    assembler
        .apply(config, LogicalEvent::StreamStop)
        .expect("stream stop");
}

fn spilled_path(assembler: &TransactionAssembler) -> std::path::PathBuf {
    assembler
        .streamed
        .get("42")
        .expect("streamed transaction")
        .changes
        .spilled_path()
        .expect("spill path")
}
