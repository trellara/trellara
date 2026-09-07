use super::*;

pub(in crate::tests) async fn publish_local_strict_chunked_transaction(
    config: &TrellaraConfig,
    transaction_id: &str,
) -> LocalPublisher {
    let publisher = new_local_publisher(config);
    let envelope = local_two_change_envelope(config, transaction_id, "0/16B6D00");
    let strict_chunking = config
        .dataset
        .strict_chunking
        .as_ref()
        .expect("strict chunking config");
    let plan = trellara_protocol::plan_strict_chunked_transaction(
        &envelope,
        &trellara_protocol::StrictChunkPlanConfig {
            max_changes_per_chunk: strict_chunking.max_changes_per_chunk,
        },
    )
    .expect("strict chunk plan");
    publisher
        .publish(
            trellara_stream::StreamMessage::transaction_manifest(&envelope, &plan.manifest)
                .expect("manifest message"),
        )
        .await
        .expect("publish manifest");
    shared::publish_commit_marker(&publisher, &envelope, &plan.manifest).await;
    for chunk in &plan.chunks {
        publisher
            .publish(
                trellara_stream::StreamMessage::strict_chunk(&envelope, chunk)
                    .expect("strict chunk message"),
            )
            .await
            .expect("publish strict chunk");
    }
    publisher
}
