use super::*;
use crate::pgoutput_stream_state::PgOutputStreamState;

#[test]
fn stream_state_rejects_nested_stream_start() {
    let mut state = PgOutputStreamState::default();

    state.start(42, true).expect("first stream start");

    assert!(matches!(
        state.start(43, true),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream start for xid 43")
                && message.contains("stream xid 42 is still active")
    ));
}

#[test]
fn stream_state_requires_active_stream_for_stop() {
    let mut state = PgOutputStreamState::default();

    assert!(matches!(
        state.stop(),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream stop arrived without an active stream")
    ));
}

#[test]
fn stream_state_rejects_commit_before_stop() {
    let mut state = PgOutputStreamState::default();

    state.start(42, true).expect("stream start");

    assert!(matches!(
        state.commit(42),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream commit for xid 42 arrived before stream stop")
    ));
}

#[test]
fn stream_state_rejects_unknown_stream_commit() {
    let mut state = PgOutputStreamState::default();

    assert!(matches!(
        state.commit(42),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream commit for xid 42")
                && message.contains("without an open streamed transaction")
    ));
}

#[test]
fn stream_state_rejects_non_first_segment_without_open_transaction() {
    let mut state = PgOutputStreamState::default();

    assert!(matches!(
        state.start(42, false),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("non-first stream segment for xid 42")
                && message.contains("before a first segment")
    ));
}

#[test]
fn stream_state_rejects_duplicate_first_segment_after_stop() {
    let mut state = PgOutputStreamState::default();

    state.start(42, true).expect("stream start");
    state.stop().expect("stream stop");

    assert!(matches!(
        state.start(42, true),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("first stream segment for xid 42")
                && message.contains("already open")
    ));
}

#[test]
fn stream_state_allows_interleaved_open_streamed_transactions() {
    let mut state = PgOutputStreamState::default();

    state.start(42, true).expect("first transaction segment");
    state.stop().expect("first transaction stop");
    state.start(43, true).expect("second transaction segment");
    state.stop().expect("second transaction stop");

    state.commit(42).expect("first transaction commit");
    state.commit(43).expect("second transaction commit");
}

#[test]
fn stream_state_parent_abort_clears_active_stream() {
    let mut state = PgOutputStreamState::default();

    state.start(42, true).expect("stream start");
    state.abort(42, 42).expect("parent abort");

    state
        .start(43, true)
        .expect("new stream may start after abort");
}

#[test]
fn stream_state_subtransaction_abort_keeps_parent_active() {
    let mut state = PgOutputStreamState::default();

    state.start(42, true).expect("stream start");
    state.abort(42, 43).expect("subtransaction abort");

    assert!(matches!(
        state.start(44, true),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream start for xid 44")
                && message.contains("stream xid 42 is still active")
    ));
}

#[test]
fn stream_state_rejects_abort_for_other_active_stream() {
    let mut state = PgOutputStreamState::default();

    state.start(42, true).expect("stream start");

    assert!(matches!(
        state.abort(43, 43),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream abort for xid 43")
                && message.contains("stream xid 42 is still active")
    ));
}

#[test]
fn stream_state_rejects_abort_without_open_transaction() {
    let mut state = PgOutputStreamState::default();

    assert!(matches!(
        state.abort(42, 42),
        Err(CaptureError::PgOutputParse(message))
            if message.contains("stream abort for xid 42")
                && message.contains("without an open streamed transaction")
    ));
}
