use rdkafka::message::{BorrowedMessage, Header, Headers, Message, OwnedHeaders};
use trellara_stream::{StreamHeader, StreamMessage, StreamPosition};

pub(crate) fn stream_message_from_kafka(message: &BorrowedMessage<'_>) -> StreamMessage {
    StreamMessage {
        topic: message.topic().to_string(),
        key: message
            .key()
            .map(|key| String::from_utf8_lossy(key).into_owned())
            .unwrap_or_default(),
        payload: message.payload().unwrap_or_default().to_vec().into(),
        headers: message
            .headers()
            .map(headers_from_kafka)
            .unwrap_or_default(),
        partition: Some(message.partition()),
        position: Some(StreamPosition {
            topic: message.topic().to_string(),
            partition: message.partition(),
            offset: message.offset(),
        }),
    }
}

fn headers_from_kafka(headers: &impl Headers) -> Vec<StreamHeader> {
    headers
        .iter()
        .map(|header| {
            StreamHeader::new(
                header.key,
                header
                    .value
                    .map(|value| String::from_utf8_lossy(value).into_owned())
                    .unwrap_or_default(),
            )
        })
        .collect()
}

pub(crate) fn owned_headers(message: &StreamMessage) -> OwnedHeaders {
    message
        .headers
        .iter()
        .fold(OwnedHeaders::new(), |headers, header| {
            headers.insert(Header {
                key: &header.key,
                value: Some(header.value.as_bytes()),
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdkafka::message::Headers;

    #[test]
    fn headers_are_copied_into_owned_headers() {
        let message = StreamMessage {
            topic: "trellara.source.dataset.strict".to_string(),
            key: "key".to_string(),
            payload: Vec::new().into(),
            headers: vec![
                StreamHeader::new("trellara.source_id", "source"),
                StreamHeader::new("trellara.commit_lsn", "0/16B6C50"),
            ],
            partition: Some(0),
            position: None,
        };

        let headers = owned_headers(&message);
        assert_eq!(headers.count(), 2);
    }
}
