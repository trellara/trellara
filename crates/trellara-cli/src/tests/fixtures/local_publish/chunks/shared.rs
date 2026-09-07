use super::*;

pub(super) fn partitioned_publish_plan(
    config: &TrellaraConfig,
    transaction_id: &str,
) -> (
    LocalPublisher,
    TransactionEnvelope,
    trellara_protocol::PartitionPlan,
) {
    let publisher = new_local_publisher(config);
    let envelope = local_two_change_envelope(config, transaction_id, "0/16B6C90");
    let partition = config.dataset.partition.as_ref().expect("partition config");
    let plan = trellara_protocol::plan_partitioned_transaction(
        &envelope,
        &PartitionPlanConfig {
            partition_count: partition.partition_count,
            key_column: partition.key_column.clone(),
            null_key_policy: ProtocolPartitionNullKeyPolicy::Quarantine,
            key_change_policy: ProtocolPartitionKeyChangePolicy::Quarantine,
        },
    )
    .expect("partition plan");

    (publisher, envelope, plan)
}

pub(super) async fn publish_partitioned_barrier(
    publisher: &LocalPublisher,
    envelope: &TransactionEnvelope,
    plan: &trellara_protocol::PartitionPlan,
) {
    publish_partition_manifest(publisher, envelope, plan).await;
    publish_commit_marker(publisher, envelope, &plan.manifest).await;
}

pub(super) async fn publish_partition_manifest(
    publisher: &LocalPublisher,
    envelope: &TransactionEnvelope,
    plan: &trellara_protocol::PartitionPlan,
) {
    publisher
        .publish(
            trellara_stream::StreamMessage::transaction_manifest(envelope, &plan.manifest)
                .expect("manifest message"),
        )
        .await
        .expect("publish manifest");
}

pub(super) async fn publish_commit_marker(
    publisher: &LocalPublisher,
    envelope: &TransactionEnvelope,
    manifest: &trellara_protocol::TransactionManifest,
) {
    let marker =
        trellara_protocol::TransactionCommitMarker::from_manifest(manifest).expect("commit marker");
    publisher
        .publish(
            trellara_stream::StreamMessage::commit_marker(envelope, &marker)
                .expect("commit marker message"),
        )
        .await
        .expect("publish commit marker");
}

pub(super) fn replace_header(message: &mut trellara_stream::StreamMessage, key: &str, value: &str) {
    let header = message
        .headers
        .iter_mut()
        .find(|header| header.key == key)
        .expect("header");
    header.value = value.to_string();
}
