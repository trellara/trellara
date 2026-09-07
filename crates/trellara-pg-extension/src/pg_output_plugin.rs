use pgrx::pg_sys;

use crate::{NativeLogicalFrameBuilder, NativeLogicalOperation};

mod change;
mod message;
mod stream;
mod tuple;
mod twophase;

use change::{
    append_change, decoder_state, or_error, relation_identity, transaction_ref,
    write_committed_frame,
};
use message::{message, stream_message};
use stream::{
    stream_abort, stream_change, stream_commit, stream_start, stream_stop, stream_truncate,
};
use twophase::{
    begin_prepare, commit_prepared, filter_prepare, prepare, rollback_prepared, stream_prepare,
};

pub(super) struct DecoderState {
    frame: NativeLogicalFrameBuilder,
}

#[pgrx::pg_guard]
#[unsafe(no_mangle)]
pub unsafe extern "C-unwind" fn _PG_output_plugin_init(
    callbacks: *mut pg_sys::OutputPluginCallbacks,
) {
    if callbacks.is_null() {
        pgrx::error!("Trellara output plugin received a null callback table");
    }
    unsafe {
        (*callbacks).startup_cb = Some(startup);
        (*callbacks).begin_cb = Some(begin);
        (*callbacks).change_cb = Some(change);
        (*callbacks).truncate_cb = Some(truncate);
        (*callbacks).commit_cb = Some(commit);
        (*callbacks).message_cb = Some(message);
        (*callbacks).filter_by_origin_cb = Some(filter_by_origin);
        (*callbacks).shutdown_cb = Some(shutdown);
        (*callbacks).filter_prepare_cb = Some(filter_prepare);
        (*callbacks).begin_prepare_cb = Some(begin_prepare);
        (*callbacks).prepare_cb = Some(prepare);
        (*callbacks).commit_prepared_cb = Some(commit_prepared);
        (*callbacks).rollback_prepared_cb = Some(rollback_prepared);
        (*callbacks).stream_start_cb = Some(stream_start);
        (*callbacks).stream_stop_cb = Some(stream_stop);
        (*callbacks).stream_abort_cb = Some(stream_abort);
        (*callbacks).stream_prepare_cb = Some(stream_prepare);
        (*callbacks).stream_commit_cb = Some(stream_commit);
        (*callbacks).stream_change_cb = Some(stream_change);
        (*callbacks).stream_message_cb = Some(stream_message);
        (*callbacks).stream_truncate_cb = Some(stream_truncate);
    }
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn startup(
    context: *mut pg_sys::LogicalDecodingContext,
    options: *mut pg_sys::OutputPluginOptions,
    _is_init: bool,
) {
    if context.is_null() || options.is_null() {
        pgrx::error!("Trellara output plugin startup received a null PostgreSQL context");
    }
    unsafe {
        (*options).output_type = pg_sys::OutputPluginOutputType::OUTPUT_PLUGIN_BINARY_OUTPUT;
        (*options).receive_rewrites = false;
        (*context).output_plugin_private = Box::into_raw(Box::new(DecoderState {
            frame: NativeLogicalFrameBuilder::default(),
        }))
        .cast();
    }
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn begin(
    context: *mut pg_sys::LogicalDecodingContext,
    transaction: *mut pg_sys::ReorderBufferTXN,
) {
    let state = unsafe { decoder_state(context) };
    if state.frame.is_active() {
        state.frame.abort();
    }
    let transaction = unsafe { transaction_ref(transaction) };
    or_error(state.frame.begin(
        transaction.xid.into_inner(),
        transaction.first_lsn,
        &crate::pg_guc::source_id(),
        &crate::pg_guc::dataset_id(),
    ));
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn change(
    context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    relation: pg_sys::Relation,
    change: *mut pg_sys::ReorderBufferChange,
) {
    unsafe { append_change(decoder_state(context), relation, change) };
}

#[pgrx::pg_guard]
pub(super) unsafe extern "C-unwind" fn truncate(
    context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    relation_count: std::ffi::c_int,
    relations: *mut pg_sys::Relation,
    _change: *mut pg_sys::ReorderBufferChange,
) {
    if relation_count < 0 || (relation_count > 0 && relations.is_null()) {
        pgrx::error!("Trellara output plugin received invalid truncate relations");
    }
    let state = unsafe { decoder_state(context) };
    for index in 0..relation_count {
        let relation = unsafe { *relations.add(index as usize) };
        let (oid, namespace, relation_name) = unsafe { relation_identity(relation) };
        or_error(state.frame.append_change_start(
            NativeLogicalOperation::Truncate,
            oid,
            &namespace,
            &relation_name,
        ));
        or_error(state.frame.append_tuple_presence(false));
        or_error(state.frame.append_tuple_presence(false));
        or_error(state.frame.finish_event());
    }
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn commit(
    context: *mut pg_sys::LogicalDecodingContext,
    _transaction: *mut pg_sys::ReorderBufferTXN,
    commit_lsn: pg_sys::XLogRecPtr,
) {
    unsafe { write_committed_frame(context, commit_lsn) };
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn filter_by_origin(
    _context: *mut pg_sys::LogicalDecodingContext,
    origin_id: pg_sys::RepOriginId,
) -> bool {
    origin_id != pg_sys::InvalidRepOriginId as pg_sys::RepOriginId
}

#[pgrx::pg_guard]
unsafe extern "C-unwind" fn shutdown(context: *mut pg_sys::LogicalDecodingContext) {
    if context.is_null() {
        return;
    }
    let private = unsafe { (*context).output_plugin_private };
    if !private.is_null() {
        unsafe {
            drop(Box::from_raw(private.cast::<DecoderState>()));
            (*context).output_plugin_private = std::ptr::null_mut();
        }
    }
}
