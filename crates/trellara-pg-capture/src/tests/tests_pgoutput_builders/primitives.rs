pub(super) fn put_cstr(message: &mut Vec<u8>, value: &str) {
    message.extend_from_slice(value.as_bytes());
    message.push(0);
}

pub(super) fn put_sized(message: &mut Vec<u8>, value: &[u8]) {
    put_i32(message, value.len() as i32);
    message.extend_from_slice(value);
}

pub(super) fn put_u16(message: &mut Vec<u8>, value: u16) {
    message.extend_from_slice(&value.to_be_bytes());
}

pub(super) fn put_u32(message: &mut Vec<u8>, value: u32) {
    message.extend_from_slice(&value.to_be_bytes());
}

pub(super) fn put_u64(message: &mut Vec<u8>, value: u64) {
    message.extend_from_slice(&value.to_be_bytes());
}

pub(super) fn put_i32(message: &mut Vec<u8>, value: i32) {
    message.extend_from_slice(&value.to_be_bytes());
}

pub(super) fn put_i64(message: &mut Vec<u8>, value: i64) {
    message.extend_from_slice(&value.to_be_bytes());
}
