use std::path::Path;

use crc32fast::Hasher;

use crate::{LocalStreamError, Result};

pub(crate) fn frame_checksum(bytes: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finalize()
}

pub(crate) fn verify_frame_checksum(
    path: &Path,
    body: &[u8],
    expected_checksum: u32,
) -> Result<()> {
    let actual_checksum = frame_checksum(body);
    if actual_checksum == expected_checksum {
        Ok(())
    } else {
        Err(LocalStreamError::CorruptFrame {
            path: path.display().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_frame_checksum_accepts_matching_checksum() {
        let body = b"record body";
        let path = Path::new("stream.log");

        verify_frame_checksum(path, body, frame_checksum(body)).expect("matching checksum");
    }

    #[test]
    fn verify_frame_checksum_rejects_mismatched_checksum() {
        let body = b"record body";
        let path = Path::new("stream.log");

        assert!(matches!(
            verify_frame_checksum(path, body, frame_checksum(body) + 1),
            Err(LocalStreamError::CorruptFrame { .. })
        ));
    }
}
