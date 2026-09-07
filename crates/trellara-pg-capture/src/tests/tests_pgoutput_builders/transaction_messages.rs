use super::primitives::{put_cstr, put_i32, put_i64, put_u16, put_u32, put_u64};
use super::tuple::{put_tuple, TupleValue};

pub(in crate::tests) fn pgoutput_begin_message(
    final_lsn: u64,
    xid: u32,
    timestamp_micros: i64,
) -> Vec<u8> {
    let mut message = vec![b'B'];
    put_u64(&mut message, final_lsn);
    put_i64(&mut message, timestamp_micros);
    put_u32(&mut message, xid);
    message
}

pub(in crate::tests) fn pgoutput_commit_message(
    commit_lsn: u64,
    end_lsn: u64,
    timestamp_micros: i64,
) -> Vec<u8> {
    let mut message = vec![b'C', 0];
    put_u64(&mut message, commit_lsn);
    put_u64(&mut message, end_lsn);
    put_i64(&mut message, timestamp_micros);
    message
}

pub(in crate::tests) fn pgoutput_relation_message(
    oid: u32,
    schema: &str,
    table: &str,
    replica_identity: u8,
    columns: &[(&str, u32, bool)],
) -> Vec<u8> {
    let mut message = vec![b'R'];
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

pub(in crate::tests) fn pgoutput_insert_message(oid: u32, values: &[TupleValue<'_>]) -> Vec<u8> {
    let mut message = vec![b'I'];
    put_u32(&mut message, oid);
    message.push(b'N');
    put_tuple(&mut message, values);
    message
}

pub(in crate::tests) fn pgoutput_update_message(
    oid: u32,
    before: Option<(u8, Vec<TupleValue<'_>>)>,
    after: &[TupleValue<'_>],
) -> Vec<u8> {
    let mut message = vec![b'U'];
    put_u32(&mut message, oid);
    if let Some((tag, values)) = before {
        message.push(tag);
        put_tuple(&mut message, &values);
    }
    message.push(b'N');
    put_tuple(&mut message, after);
    message
}

pub(in crate::tests) fn pgoutput_delete_message(
    oid: u32,
    tuple_tag: u8,
    values: &[TupleValue<'_>],
) -> Vec<u8> {
    let mut message = vec![b'D'];
    put_u32(&mut message, oid);
    message.push(tuple_tag);
    put_tuple(&mut message, values);
    message
}

pub(in crate::tests) fn pgoutput_truncate_message(oids: &[u32]) -> Vec<u8> {
    let mut message = vec![b'T'];
    put_u32(&mut message, oids.len() as u32);
    message.push(0);
    for oid in oids {
        put_u32(&mut message, *oid);
    }
    message
}
