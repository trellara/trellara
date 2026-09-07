use super::*;

#[tokio::test]
async fn local_publish_rejects_empty_record_key() {
    let root = temp_root("publish-empty-key");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");

    let error = publisher
        .publish(message("trellara.source.dataset.strict", " ", "payload"))
        .await
        .expect_err("empty key");

    assert!(matches!(error, StreamError::Publisher(reason) if reason.contains("field key")));
    fs::remove_dir_all(root).expect("cleanup");
}

#[tokio::test]
async fn local_publish_rejects_duplicate_header_keys() {
    let root = temp_root("publish-duplicate-headers");
    let publisher = LocalPublisher::new(LocalPublisherConfig::new(&root)).expect("publisher");
    let mut duplicate = message("trellara.source.dataset.strict", "tx-1", "payload");
    duplicate
        .headers
        .push(StreamHeader::new("trellara.transaction_id", "tx-1"));

    let error = publisher
        .publish(duplicate)
        .await
        .expect_err("duplicate header");

    assert!(matches!(error, StreamError::Publisher(reason) if reason.contains("header.key")));
    fs::remove_dir_all(root).expect("cleanup");
}
