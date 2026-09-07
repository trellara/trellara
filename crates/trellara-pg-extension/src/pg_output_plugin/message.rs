use std::{ffi::CStr, slice};

use pgrx::pg_sys;

use super::change::{decoder_state, or_error, transaction_ref, write_committed_frame};

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn message(
    context: *mut pg_sys::LogicalDecodingContext,
    transaction: *mut pg_sys::ReorderBufferTXN,
    message_lsn: pg_sys::XLogRecPtr,
    transactional: bool,
    prefix: *const std::ffi::c_char,
    message_size: pg_sys::Size,
    message: *const std::ffi::c_char,
) {
    unsafe {
        append_message(
            context,
            transaction,
            message_lsn,
            transactional,
            prefix,
            message_size,
            message,
        )
    };
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn stream_message(
    context: *mut pg_sys::LogicalDecodingContext,
    transaction: *mut pg_sys::ReorderBufferTXN,
    message_lsn: pg_sys::XLogRecPtr,
    transactional: bool,
    prefix: *const std::ffi::c_char,
    message_size: pg_sys::Size,
    message: *const std::ffi::c_char,
) {
    unsafe {
        append_message(
            context,
            transaction,
            message_lsn,
            transactional,
            prefix,
            message_size,
            message,
        )
    };
}

unsafe fn append_message(
    context: *mut pg_sys::LogicalDecodingContext,
    transaction: *mut pg_sys::ReorderBufferTXN,
    message_lsn: pg_sys::XLogRecPtr,
    transactional: bool,
    prefix: *const std::ffi::c_char,
    message_size: pg_sys::Size,
    message: *const std::ffi::c_char,
) {
    if prefix.is_null() || (message_size > 0 && message.is_null()) {
        pgrx::error!("Trellara output plugin received an invalid logical message");
    }
    let prefix = unsafe { CStr::from_ptr(prefix) }.to_bytes();
    let payload = if message_size == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(message.cast::<u8>(), message_size) }
    };

    if transactional {
        if !unsafe { decoder_state(context) }.frame.is_active() {
            unsafe { super::begin(context, transaction) };
        }
        let state = unsafe { decoder_state(context) };
        or_error(
            state
                .frame
                .append_message(message_lsn, true, prefix, payload),
        );
        or_error(state.frame.finish_event());
        return;
    }

    let state = unsafe { decoder_state(context) };
    if state.frame.is_active() {
        pgrx::error!(
            "Trellara output plugin cannot emit a non-transactional logical message while a transaction frame is active"
        );
    }
    let xid = if transaction.is_null() {
        0
    } else {
        unsafe { transaction_ref(transaction) }.xid.into_inner()
    };
    or_error(state.frame.begin(
        xid,
        message_lsn,
        &crate::pg_guc::source_id(),
        &crate::pg_guc::dataset_id(),
    ));
    or_error(
        state
            .frame
            .append_message(message_lsn, false, prefix, payload),
    );
    or_error(state.frame.finish_event());
    unsafe { write_committed_frame(context, message_lsn) };
}
