use std::io;

use bytes::{Buf, BytesMut};

const MAX_BACKEND_MESSAGE_BODY_BYTES: usize = 64 * 1024 * 1024;

pub(crate) struct RawBackendMessage {
    pub(crate) tag: u8,
    pub(crate) body: Vec<u8>,
}

impl RawBackendMessage {
    pub(crate) fn parse(buf: &mut BytesMut) -> io::Result<Option<Self>> {
        if buf.len() < 5 {
            return Ok(None);
        }
        let tag = buf[0];
        let len = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        let body_len = backend_message_body_len(len)?;
        let total_len = body_len + 5;
        if buf.len() < total_len {
            return Ok(None);
        }
        buf.advance(5);
        let body = buf.split_to(body_len).to_vec();
        Ok(Some(Self { tag, body }))
    }
}

fn backend_message_body_len(len: i32) -> io::Result<usize> {
    if len < 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid backend message length",
        ));
    }
    let body_len = usize::try_from(len - 4).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "backend message length exceeds supported platform range",
        )
    })?;
    if body_len > MAX_BACKEND_MESSAGE_BODY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "backend message body length {body_len} exceeds {} byte safety limit",
                MAX_BACKEND_MESSAGE_BODY_BYTES
            ),
        ));
    }
    Ok(body_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_message_body_len_rejects_invalid_wire_lengths() {
        let error = backend_message_body_len(3).expect_err("invalid length");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("invalid backend message length"));
    }

    #[test]
    fn backend_message_body_len_rejects_oversized_frames() {
        let oversized_len =
            i32::try_from(MAX_BACKEND_MESSAGE_BODY_BYTES + 5).expect("test size fits i32");
        let error = backend_message_body_len(oversized_len).expect_err("oversized frame");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("exceeds"));
    }

    #[test]
    fn raw_backend_message_parse_waits_for_complete_body() {
        let mut buf = BytesMut::from(&[b'W', 0, 0, 0, 8, b'a'][..]);

        assert!(RawBackendMessage::parse(&mut buf)
            .expect("partial frame")
            .is_none());
        assert_eq!(buf.as_ref(), &[b'W', 0, 0, 0, 8, b'a']);
    }

    #[test]
    fn raw_backend_message_parse_extracts_complete_body() {
        let mut buf = BytesMut::from(&[b'W', 0, 0, 0, 8, b'a', b'b', b'c', b'd', b'Z'][..]);

        let message = RawBackendMessage::parse(&mut buf)
            .expect("frame parse")
            .expect("complete frame");

        assert_eq!(message.tag, b'W');
        assert_eq!(message.body, b"abcd");
        assert_eq!(buf.as_ref(), b"Z");
    }
}
