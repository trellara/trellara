use rdkafka::TopicPartitionList;
use trellara_stream::StreamPosition;

use crate::KafkaStreamError;

pub(crate) fn ack_offsets(
    position: &StreamPosition,
) -> Result<TopicPartitionList, KafkaStreamError> {
    validate_ack_position(position)?;
    let mut offsets = TopicPartitionList::new();
    offsets.add_partition_offset(
        &position.topic,
        position.partition,
        rdkafka::Offset::Offset(position.offset.saturating_add(1)),
    )?;
    Ok(offsets)
}

fn validate_ack_position(position: &StreamPosition) -> Result<(), KafkaStreamError> {
    if position.partition < 0 {
        return Err(KafkaStreamError::InvalidStreamPosition {
            field: "partition",
            value: i64::from(position.partition),
            reason: "must be non-negative",
        });
    }
    if position.offset < 0 {
        return Err(KafkaStreamError::InvalidStreamPosition {
            field: "offset",
            value: position.offset,
            reason: "must be non-negative",
        });
    }
    if position.offset == i64::MAX {
        return Err(KafkaStreamError::InvalidStreamPosition {
            field: "offset",
            value: position.offset,
            reason: "cannot advance beyond i64::MAX",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ack_offsets_commit_next_offset() {
        let offsets = ack_offsets(&StreamPosition {
            topic: "trellara.source.dataset.strict".to_string(),
            partition: 3,
            offset: 41,
        })
        .expect("offsets");

        let elements = offsets.elements();
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].topic(), "trellara.source.dataset.strict");
        assert_eq!(elements[0].partition(), 3);
        assert_eq!(elements[0].offset(), rdkafka::Offset::Offset(42));
    }

    #[test]
    fn ack_offsets_reject_invalid_stream_positions() {
        for (field, position) in [
            (
                "partition",
                StreamPosition {
                    topic: "trellara.source.dataset.strict".to_string(),
                    partition: -1,
                    offset: 41,
                },
            ),
            (
                "offset",
                StreamPosition {
                    topic: "trellara.source.dataset.strict".to_string(),
                    partition: 3,
                    offset: -1,
                },
            ),
            (
                "offset",
                StreamPosition {
                    topic: "trellara.source.dataset.strict".to_string(),
                    partition: 3,
                    offset: i64::MAX,
                },
            ),
        ] {
            assert!(matches!(
                ack_offsets(&position),
                Err(KafkaStreamError::InvalidStreamPosition {
                    field: actual,
                    ..
                }) if actual == field
            ));
        }
    }
}
