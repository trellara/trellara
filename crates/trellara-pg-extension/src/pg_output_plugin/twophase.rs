use pgrx::pg_sys;

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn filter_prepare(
    _context: *mut pg_sys::LogicalDecodingContext,
    _xid: pg_sys::TransactionId,
    _gid: *const std::ffi::c_char,
) -> bool {
    unsupported_two_phase("filter_prepare_cb")
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn begin_prepare(
    _context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
) {
    unsupported_two_phase("begin_prepare_cb")
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn prepare(
    _context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    _prepare_lsn: pg_sys::XLogRecPtr,
) {
    unsupported_two_phase("prepare_cb")
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn commit_prepared(
    _context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    _commit_lsn: pg_sys::XLogRecPtr,
) {
    unsupported_two_phase("commit_prepared_cb")
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn rollback_prepared(
    _context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    _prepare_end_lsn: pg_sys::XLogRecPtr,
    _prepare_time: pg_sys::TimestampTz,
) {
    unsupported_two_phase("rollback_prepared_cb")
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn stream_prepare(
    _context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    _prepare_lsn: pg_sys::XLogRecPtr,
) {
    unsupported_two_phase("stream_prepare_cb")
}

fn unsupported_two_phase(callback: &'static str) -> ! {
    pgrx::error!(
        "Trellara native output plugin does not support two-phase commit yet; {callback} failed closed"
    )
}
