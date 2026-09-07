use pgrx::pg_sys;

use super::change::{append_change, decoder_state, write_committed_frame};

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn stream_start(
    context: *mut pg_sys::LogicalDecodingContext,
    transaction: *mut pg_sys::ReorderBufferTXN,
) {
    let state = unsafe { decoder_state(context) };
    if !state.frame.is_active() {
        unsafe { super::begin(context, transaction) };
    }
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn stream_stop(
    _context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
) {
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn stream_abort(
    context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    _abort_lsn: pg_sys::XLogRecPtr,
) {
    unsafe { decoder_state(context) }.frame.abort();
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn stream_commit(
    context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    commit_lsn: pg_sys::XLogRecPtr,
) {
    unsafe { write_committed_frame(context, commit_lsn) };
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn stream_change(
    context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    relation: pg_sys::Relation,
    change: *mut pg_sys::ReorderBufferChange,
) {
    unsafe { append_change(decoder_state(context), relation, change) };
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn stream_truncate(
    context: *mut pg_sys::LogicalDecodingContext,
    transaction: *mut pg_sys::ReorderBufferTXN,
    relation_count: std::ffi::c_int,
    relations: *mut pg_sys::Relation,
    change: *mut pg_sys::ReorderBufferChange,
) {
    unsafe { super::truncate(context, transaction, relation_count, relations, change) };
}
