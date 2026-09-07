use super::*;

pub(in crate::tests) async fn publish_local_partitioned_transaction(
    config: &TrellaraConfig,
    transaction_id: &str,
    omit_last_chunk: bool,
) -> LocalPublisher {
    let (publisher, envelope, plan) = shared::partitioned_publish_plan(config, transaction_id);
    shared::publish_partitioned_barrier(&publisher, &envelope, &plan).await;
    let chunk_count = plan.chunks.len();
    for (index, chunk) in plan.chunks.iter().enumerate() {
        if omit_last_chunk && index + 1 == chunk_count {
            continue;
        }
        publisher
            .publish(
                trellara_stream::StreamMessage::partition_chunk(&envelope, chunk)
                    .expect("partition chunk message"),
            )
            .await
            .expect("publish partition chunk");
    }
    publisher
}

pub(in crate::tests) async fn publish_local_partitioned_transaction_with_conflicting_commit_count(
    config: &TrellaraConfig,
    transaction_id: &str,
) -> LocalPublisher {
    let (publisher, envelope, plan) = shared::partitioned_publish_plan(config, transaction_id);
    shared::publish_partition_manifest(&publisher, &envelope, &plan).await;
    let marker = trellara_protocol::TransactionCommitMarker::from_manifest(&plan.manifest)
        .expect("commit marker");
    let mut commit_message = trellara_stream::StreamMessage::commit_marker(&envelope, &marker)
        .expect("commit marker message");
    shared::replace_header(
        &mut commit_message,
        "trellara.partition_count",
        &(marker.participating_partition_count + 1).to_string(),
    );
    publisher
        .publish(commit_message)
        .await
        .expect("publish commit marker");
    for chunk in &plan.chunks {
        publisher
            .publish(
                trellara_stream::StreamMessage::partition_chunk(&envelope, chunk)
                    .expect("partition chunk message"),
            )
            .await
            .expect("publish partition chunk");
    }
    publisher
}

pub(in crate::tests) async fn publish_local_partitioned_transaction_with_conflicting_source_id(
    config: &TrellaraConfig,
    transaction_id: &str,
) -> LocalPublisher {
    let (publisher, envelope, plan) = shared::partitioned_publish_plan(config, transaction_id);
    shared::publish_partitioned_barrier(&publisher, &envelope, &plan).await;
    for (index, chunk) in plan.chunks.iter().enumerate() {
        let mut chunk_message = trellara_stream::StreamMessage::partition_chunk(&envelope, chunk)
            .expect("partition chunk message");
        if index == 0 {
            shared::replace_header(&mut chunk_message, "trellara.source_id", "wrong-source");
        }
        publisher
            .publish(chunk_message)
            .await
            .expect("publish partition chunk");
    }
    publisher
}
