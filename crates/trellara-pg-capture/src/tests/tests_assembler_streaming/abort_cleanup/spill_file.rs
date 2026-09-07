use super::*;
use std::fs;

#[test]
fn assembler_removes_spill_file_after_stream_abort() {
    let spill_dir = stream_spill_test_dir("abort-cleanup");
    let mut assembler = TransactionAssembler::with_stream_spill_config(1, Some(spill_dir.clone()));
    let config = assembler_config();

    assembler
        .apply(
            &config,
            LogicalEvent::StreamStart {
                transaction_id: "42".to_string(),
                first_segment: true,
            },
        )
        .expect("stream start");
    assembler
        .apply(&config, streamed_insert_event("42", "sale-1"))
        .expect("first streamed change");
    assembler
        .apply(&config, streamed_insert_event("42", "sale-2"))
        .expect("second streamed change");
    let spill_path = assembler
        .streamed
        .get("42")
        .expect("streamed transaction")
        .changes
        .spilled_path()
        .expect("spill path");
    assert!(spill_path.exists());

    assembler
        .apply(
            &config,
            LogicalEvent::StreamAbort {
                transaction_id: "42".to_string(),
                subtransaction_id: "42".to_string(),
            },
        )
        .expect("stream abort");

    assert!(!assembler.streamed.contains_key("42"));
    assert!(!spill_path.exists());
    fs::remove_dir_all(spill_dir).expect("remove spill test dir");
}
