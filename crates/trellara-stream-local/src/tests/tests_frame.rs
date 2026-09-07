use super::*;
use std::path::Path;

use crate::frame_limits::MAX_FRAME_BODY_BYTES;

#[test]
fn read_frame_rejects_absurd_body_length_before_allocation() {
    let path = Path::new("stream.log");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(FRAME_MAGIC);
    bytes.extend_from_slice(&(MAX_FRAME_BODY_BYTES + 1).to_le_bytes());
    let mut reader = bytes.as_slice();

    assert!(matches!(
        read_frame(&mut reader, path),
        Err(LocalStreamError::CorruptFrame { .. })
    ));
}
