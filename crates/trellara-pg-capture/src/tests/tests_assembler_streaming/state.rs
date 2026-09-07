use crate::assembler_spill_config::StreamSpillConfig;
use crate::assembler_stream::StreamedTransactions;
use crate::CaptureError;

#[test]
fn streamed_transactions_rejects_resume_without_first_segment() {
    let mut streamed = StreamedTransactions::default();

    assert!(matches!(
        streamed.start("42".to_string(), false, &StreamSpillConfig::default()),
        Err(CaptureError::ChangeOutsideTransaction)
    ));
}

#[test]
fn streamed_transactions_rejects_duplicate_first_segment() {
    let mut streamed = StreamedTransactions::default();

    streamed
        .start("42".to_string(), true, &StreamSpillConfig::default())
        .expect("stream start");

    assert!(matches!(
        streamed.start("42".to_string(), true, &StreamSpillConfig::default()),
        Err(CaptureError::StreamStartBeforeStop {
            transaction_id,
            active_transaction_id,
        }) if transaction_id == "42" && active_transaction_id == "42"
    ));
}

#[test]
fn streamed_transactions_rejects_duplicate_first_segment_after_stop() {
    let mut streamed = StreamedTransactions::default();

    streamed
        .start("42".to_string(), true, &StreamSpillConfig::default())
        .expect("stream start");
    streamed.stop().expect("stream stop");

    assert!(matches!(
        streamed.start("42".to_string(), true, &StreamSpillConfig::default()),
        Err(CaptureError::NestedBegin(transaction_id)) if transaction_id == "42"
    ));
}

#[test]
fn streamed_transactions_rejects_stop_without_active_transaction() {
    let mut streamed = StreamedTransactions::default();

    assert!(matches!(
        streamed.stop(),
        Err(CaptureError::StreamStopWithoutStart)
    ));
}

#[test]
fn streamed_transactions_rejects_commit_without_open_transaction() {
    let mut streamed = StreamedTransactions::default();

    assert!(matches!(
        streamed.commit("42"),
        Err(CaptureError::CommitWithoutBegin)
    ));
}

#[test]
fn streamed_transactions_rejects_commit_before_stop() {
    let mut streamed = StreamedTransactions::default();

    streamed
        .start("42".to_string(), true, &StreamSpillConfig::default())
        .expect("stream start");

    assert!(matches!(
        streamed.commit("42"),
        Err(CaptureError::StreamCommitBeforeStop {
            transaction_id,
            active_transaction_id,
        }) if transaction_id == "42" && active_transaction_id == "42"
    ));
    assert!(streamed.contains_key("42"));
}

#[test]
fn streamed_transactions_rejects_abort_without_open_parent() {
    let mut streamed = StreamedTransactions::default();

    assert!(matches!(
        streamed.abort("42", "42"),
        Err(CaptureError::ChangeOutsideTransaction)
    ));
    assert!(matches!(
        streamed.abort("42", "43"),
        Err(CaptureError::ChangeOutsideTransaction)
    ));
}
