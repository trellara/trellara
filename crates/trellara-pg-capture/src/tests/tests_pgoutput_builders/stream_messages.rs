use super::primitives::{put_cstr, put_i32, put_i64, put_u16, put_u32, put_u64};
use super::tuple::{put_tuple, TupleValue};

pub(in crate::tests) fn pgoutput_stream_relation_message(
    xid: u32,
    oid: u32,
    schema: &str,
    table: &str,
    replica_identity: u8,
    columns: &[(&str, u32, bool)],
) -> Vec<u8> {
    let mut message = vec![b'R'];
    put_u32(&mut message, xid);
    put_u32(&mut message, oid);
    put_cstr(&mut message, schema);
    put_cstr(&mut message, table);
    message.push(replica_identity);
    put_u16(&mut message, columns.len() as u16);
    for (name, type_oid, is_key) in columns {
        message.push(u8::from(*is_key));
        put_cstr(&mut message, name);
        put_u32(&mut message, *type_oid);
        put_i32(&mut message, -1);
    }
    message
}

pub(in crate::tests) fn pgoutput_stream_insert_message(
    xid: u32,
    oid: u32,
    values: &[TupleValue<'_>],
) -> Vec<u8> {
    let mut message = vec![b'I'];
    put_u32(&mut message, xid);
    put_u32(&mut message, oid);
    message.push(b'N');
    put_tuple(&mut message, values);
    message
}

pub(in crate::tests) fn pgoutput_stream_start_message(xid: u32, first_segment: bool) -> Vec<u8> {
    let mut message = vec![b'S'];
    put_u32(&mut message, xid);
    message.push(u8::from(first_segment));
    message
}

pub(in crate::tests) fn pgoutput_stream_stop_message() -> Vec<u8> {
    vec![b'E']
}

pub(in crate::tests) fn pgoutput_stream_commit_message(
    xid: u32,
    commit_lsn: u64,
    end_lsn: u64,
    timestamp_micros: i64,
) -> Vec<u8> {
    let mut message = vec![b'c'];
    put_u32(&mut message, xid);
    message.push(0);
    put_u64(&mut message, commit_lsn);
    put_u64(&mut message, end_lsn);
    put_i64(&mut message, timestamp_micros);
    message
}

pub(in crate::tests) fn pgoutput_stream_abort_message(xid: u32, subxid: u32) -> Vec<u8> {
    let mut message = vec![b'A'];
    put_u32(&mut message, xid);
    put_u32(&mut message, subxid);
    message
}
