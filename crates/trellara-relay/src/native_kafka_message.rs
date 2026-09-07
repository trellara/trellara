use trellara_pg_extension::NativeRelayWireFrame;
use trellara_protocol::format_lsn;
use trellara_stream::{StreamHeader, StreamMessage};

use crate::native_publish_proof::NativePublishDestination;

pub(crate) fn message_for_frame(
    frame: &NativeRelayWireFrame,
    destination: &NativePublishDestination,
) -> StreamMessage {
    let commit_lsn = format_lsn(frame.commit_lsn);
    StreamMessage {
        topic: destination.topic.clone(),
        key: format!(
            "{}:{}:{}:{}",
            frame.source_id, frame.dataset_id, frame.xid, commit_lsn
        ),
        payload: frame.payload.clone().into(),
        headers: vec![
            StreamHeader::new("trellara.message_kind", "native_logical_transaction"),
            StreamHeader::new("trellara.native_frame_version", "1"),
            StreamHeader::new("trellara.source_id", frame.source_id.clone()),
            StreamHeader::new("trellara.dataset_id", frame.dataset_id.clone()),
            StreamHeader::new("trellara.transaction_id", frame.xid.to_string()),
            StreamHeader::new("trellara.commit_lsn", commit_lsn),
            StreamHeader::new("trellara.payload_sha256", hex_digest(&frame.payload_digest)),
        ],
        partition: Some(destination.partition),
        position: None,
    }
}

fn hex_digest(digest: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_message_has_deterministic_destination_key_and_proof_headers() {
        let frame = NativeRelayWireFrame::new(7, 0x16_b6c50, "source", "dataset", vec![1, 2])
            .expect("frame");
        let destination = NativePublishDestination {
            cluster_id: "cluster".to_string(),
            topic: "trellara.source.dataset.strict".to_string(),
            partition: 0,
        };
        let message = message_for_frame(&frame, &destination);

        assert_eq!(message.topic, destination.topic);
        assert_eq!(message.partition, Some(0));
        assert_eq!(message.key, "source:dataset:7:0/16B6C50");
        assert!(message
            .headers
            .iter()
            .any(|header| header.key == "trellara.payload_sha256"));
    }
}
