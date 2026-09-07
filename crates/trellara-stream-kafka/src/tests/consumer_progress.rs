use rdkafka::{Offset, TopicPartitionList};
use trellara_stream::StreamPosition;

use crate::consumer_progress::{ConsumerProgress, PendingCommit};
use crate::KafkaStreamError;

fn assignment() -> TopicPartitionList {
    let mut assignment = TopicPartitionList::new();
    assignment.add_partition("events", 0);
    assignment
}

fn position(offset: i64) -> StreamPosition {
    StreamPosition {
        topic: "events".to_string(),
        partition: 0,
        offset,
    }
}

fn committed_offset(commit: &PendingCommit) -> Offset {
    commit.offsets.elements()[0].offset()
}

#[test]
fn out_of_order_apply_never_skips_an_unapplied_offset() {
    let mut progress = ConsumerProgress::default();
    progress.assign(&assignment());
    for offset in 10..=12 {
        progress
            .record_delivery(&position(offset))
            .expect("delivery");
    }

    assert!(progress
        .acknowledge(&position(12))
        .expect("ack 12")
        .is_none());
    let first = progress
        .acknowledge(&position(10))
        .expect("ack 10")
        .expect("commit 11");
    assert_eq!(committed_offset(&first), Offset::Offset(11));
    progress.commit_succeeded(first);
    let second = progress
        .acknowledge(&position(11))
        .expect("ack 11")
        .expect("commit 13");
    assert_eq!(committed_offset(&second), Offset::Offset(13));
}

#[test]
fn failed_commit_remains_retryable() {
    let mut progress = ConsumerProgress::default();
    progress.assign(&assignment());
    progress.record_delivery(&position(7)).expect("delivery");

    let first = progress
        .acknowledge(&position(7))
        .expect("first ack")
        .expect("first commit");
    assert_eq!(committed_offset(&first), Offset::Offset(8));
    let retry = progress
        .acknowledge(&position(7))
        .expect("retry ack")
        .expect("retry commit");
    assert_eq!(committed_offset(&retry), Offset::Offset(8));
}

#[test]
fn cooperative_revoke_drops_only_revoked_partition_state() {
    let mut progress = ConsumerProgress::default();
    let mut both = assignment();
    both.add_partition("events", 1);
    progress.assign(&both);
    progress
        .record_delivery(&position(1))
        .expect("partition zero");
    let mut partition_one = position(1);
    partition_one.partition = 1;
    progress
        .record_delivery(&partition_one)
        .expect("partition one");

    progress.revoke(&assignment());

    assert!(matches!(
        progress.acknowledge(&position(1)),
        Err(KafkaStreamError::PartitionNotOwned { partition: 0, .. })
    ));
    assert!(progress.acknowledge(&partition_one).is_ok());
}
