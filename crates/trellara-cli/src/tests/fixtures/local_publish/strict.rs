use super::*;

pub(in crate::tests) async fn publish_local_strict_transactions(
    config: &TrellaraConfig,
    count: usize,
) -> LocalPublisher {
    let publisher = new_local_publisher(config);
    for index in 0..count {
        let mut envelope = inspect_envelope();
        envelope.source_id = config.source.id.clone();
        envelope.database_id = config
            .source
            .database_id
            .clone()
            .unwrap_or_else(|| "postgres".to_string());
        envelope.dataset_id = config.dataset.id.clone();
        envelope.transaction_id = format!("tx-local-seek-{index}");
        envelope.begin_lsn = format!("0/{:X}", 0x16B6B00 + index as u64);
        envelope.commit_lsn = format!("0/{:X}", 0x16B6C50 + index as u64);
        for change in &mut envelope.changes {
            change.transaction_id = envelope.transaction_id.clone();
            change.idempotency_key = trellara_protocol::idempotency_key(
                &envelope.source_id,
                &envelope.commit_lsn,
                &envelope.transaction_id,
                change.total_order,
            );
        }
        envelope.manifest = None;
        envelope.finalize_checksum();
        publisher
            .publish(
                trellara_stream::StreamMessage::strict_transaction(&envelope)
                    .expect("strict stream message"),
            )
            .await
            .expect("publish local transaction");
    }
    publisher
}
