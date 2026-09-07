use std::ffi::CStr;

use pgrx::pg_sys;

use super::DecoderState;
use crate::{NativeLogicalFrameError, NativeLogicalOperation};

pub(super) unsafe fn append_change(
    state: &mut DecoderState,
    relation: pg_sys::Relation,
    change: *mut pg_sys::ReorderBufferChange,
) {
    if change.is_null() {
        pgrx::error!("Trellara output plugin received a null DML change");
    }
    let operation = match unsafe { (*change).action } {
        pg_sys::ReorderBufferChangeType::REORDER_BUFFER_CHANGE_INSERT => {
            NativeLogicalOperation::Insert
        }
        pg_sys::ReorderBufferChangeType::REORDER_BUFFER_CHANGE_UPDATE => {
            NativeLogicalOperation::Update
        }
        pg_sys::ReorderBufferChangeType::REORDER_BUFFER_CHANGE_DELETE => {
            NativeLogicalOperation::Delete
        }
        unsupported => pgrx::error!("unsupported logical DML action {unsupported}"),
    };
    let (oid, namespace, relation_name) = unsafe { relation_identity(relation) };
    or_error(
        state
            .frame
            .append_change_start(operation, oid, &namespace, &relation_name),
    );
    let tuple_change = unsafe { (*change).data.tp };
    let old_tuple = unsafe { change_tuple_heap_tuple(tuple_change.oldtuple) };
    let new_tuple = unsafe { change_tuple_heap_tuple(tuple_change.newtuple) };
    unsafe { super::tuple::append_tuple(&mut state.frame, relation, old_tuple) };
    unsafe { super::tuple::append_tuple(&mut state.frame, relation, new_tuple) };
    or_error(state.frame.finish_event());
}

#[cfg(any(feature = "pg15", feature = "pg16"))]
unsafe fn change_tuple_heap_tuple(tuple: *mut pg_sys::ReorderBufferTupleBuf) -> pg_sys::HeapTuple {
    if tuple.is_null() {
        std::ptr::null_mut()
    } else {
        unsafe { std::ptr::addr_of_mut!((*tuple).tuple) }
    }
}

#[cfg(any(feature = "pg17", feature = "pg18"))]
unsafe fn change_tuple_heap_tuple(tuple: pg_sys::HeapTuple) -> pg_sys::HeapTuple {
    tuple
}

pub(super) unsafe fn relation_identity(relation: pg_sys::Relation) -> (u32, Vec<u8>, Vec<u8>) {
    if relation.is_null() {
        pgrx::error!("Trellara output plugin received a null relation");
    }
    let oid = unsafe { (*relation).rd_id };
    let namespace = unsafe { pg_sys::get_namespace_name(pg_sys::get_rel_namespace(oid)) };
    let relation_name = unsafe { pg_sys::get_rel_name(oid) };
    if namespace.is_null() || relation_name.is_null() {
        pgrx::error!("Trellara could not resolve logical relation identity");
    }
    let namespace_bytes = unsafe { CStr::from_ptr(namespace) }.to_bytes().to_vec();
    let relation_bytes = unsafe { CStr::from_ptr(relation_name) }.to_bytes().to_vec();
    unsafe {
        pg_sys::pfree(namespace.cast());
        pg_sys::pfree(relation_name.cast());
    }
    (oid.to_u32(), namespace_bytes, relation_bytes)
}

pub(super) unsafe fn write_committed_frame(
    context: *mut pg_sys::LogicalDecodingContext,
    commit_lsn: pg_sys::XLogRecPtr,
) {
    let state = unsafe { decoder_state(context) };
    if !state.frame.has_events() {
        state.frame.abort();
        return;
    }
    let bytes = or_error(state.frame.finish(commit_lsn));
    let length = i32::try_from(bytes.len()).unwrap_or_else(|_| {
        pgrx::error!("logical frame length does not fit PostgreSQL StringInfo")
    });
    unsafe {
        pg_sys::OutputPluginPrepareWrite(context, true);
        pg_sys::appendBinaryStringInfo((*context).out, bytes.as_ptr().cast(), length);
        pg_sys::OutputPluginWrite(context, true);
    }
}

pub(super) unsafe fn decoder_state<'a>(
    context: *mut pg_sys::LogicalDecodingContext,
) -> &'a mut DecoderState {
    if context.is_null() || unsafe { (*context).output_plugin_private.is_null() } {
        pgrx::error!("Trellara output plugin decoder state is unavailable");
    }
    unsafe { &mut *(*context).output_plugin_private.cast::<DecoderState>() }
}

pub(super) unsafe fn transaction_ref<'a>(
    transaction: *mut pg_sys::ReorderBufferTXN,
) -> &'a pg_sys::ReorderBufferTXN {
    if transaction.is_null() {
        pgrx::error!("Trellara output plugin received a null transaction");
    }
    unsafe { &*transaction }
}

pub(super) fn or_error<T>(result: Result<T, NativeLogicalFrameError>) -> T {
    result.unwrap_or_else(|error| pgrx::error!("{error}"))
}
