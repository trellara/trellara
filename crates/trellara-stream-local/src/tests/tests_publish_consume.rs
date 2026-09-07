use super::*;

#[tokio::test]
async fn consumer_reads_subscribed_topics_in_order() {
    let root = temp_root("topics");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    publisher
        .publish(message(
            "trellara.source.dataset.partition.1",
            "tx-p",
            "partition",
        ))
        .await
        .expect("publish partition");
    publisher
        .publish(message(
            "trellara.source.dataset.manifest",
            "tx-m",
            "manifest",
        ))
        .await
        .expect("publish manifest");

    let mut consumer = LocalConsumer::new(LocalConsumerConfig::new(
        &root,
        "applier",
        vec![
            "trellara.source.dataset.manifest".to_string(),
            "trellara.source.dataset.partition.1".to_string(),
        ],
    ))
    .expect("consumer");

    let first = consumer.next().await.expect("next").expect("message");
    assert_eq!(first.topic, "trellara.source.dataset.manifest");
    assert_eq!(first.key, "tx-m");
    consumer.ack(&first).await.expect("ack");
    let second = consumer.next().await.expect("next").expect("message");
    assert_eq!(second.topic, "trellara.source.dataset.partition.1");
    assert_eq!(second.key, "tx-p");

    fs::remove_dir_all(root).expect("cleanup");
}
