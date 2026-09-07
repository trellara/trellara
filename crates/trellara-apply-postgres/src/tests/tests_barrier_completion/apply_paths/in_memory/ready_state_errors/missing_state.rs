use super::super::*;

#[tokio::test]
async fn ready_transaction_missing_pending_state_returns_typed_error() {
    let mut consumer = RecordingConsumer::new(Vec::new());
    let mut applier = RecordingApplier::new(Vec::new());
    let mut pending = HashMap::new();

    let error = crate::worker_transaction::apply_ready_transaction(
        &mut consumer,
        &mut applier,
        &mut pending,
        "source:0/16B7000:tx-missing",
    )
    .await
    .expect_err("missing pending state");

    assert!(matches!(
        error,
        ApplyWorkerError::ReadyTransactionMissing {
            transaction_key,
        } if transaction_key == "source:0/16B7000:tx-missing"
    ));
}

#[tokio::test]
async fn ready_transaction_missing_manifest_returns_typed_error() {
    let mut consumer = RecordingConsumer::new(Vec::new());
    let mut applier = RecordingApplier::new(Vec::new());
    let mut pending = HashMap::from([(
        "source:0/16B7000:tx-missing-manifest".to_string(),
        PendingBarrierTransaction::default(),
    )]);

    let error = crate::worker_transaction::apply_ready_transaction(
        &mut consumer,
        &mut applier,
        &mut pending,
        "source:0/16B7000:tx-missing-manifest",
    )
    .await
    .expect_err("missing manifest");

    assert!(matches!(
        error,
        ApplyWorkerError::ReadyTransactionMissingManifest {
            transaction_key,
        } if transaction_key == "source:0/16B7000:tx-missing-manifest"
    ));
}

#[tokio::test]
async fn ready_transaction_missing_commit_marker_returns_typed_error() {
    let (_chunk_message, manifest_message, _commit_message) =
        partitioned_messages(&envelope("tx-missing-commit", "0/16B7000"));
    let transaction_key = HeaderContext::from_message(&manifest_message)
        .expect("context")
        .transaction_key();
    let mut pending = HashMap::from([(
        transaction_key.clone(),
        PendingBarrierTransaction {
            manifest: Some(pending_manifest(manifest_message)),
            commit_marker: None,
            chunks: Default::default(),
        },
    )]);
    let mut consumer = RecordingConsumer::new(Vec::new());
    let mut applier = RecordingApplier::new(Vec::new());

    let error = crate::worker_transaction::apply_ready_transaction(
        &mut consumer,
        &mut applier,
        &mut pending,
        &transaction_key,
    )
    .await
    .expect_err("missing commit marker");

    assert!(matches!(
        error,
        ApplyWorkerError::ReadyTransactionMissingCommitMarker {
            transaction_key: actual,
        } if actual == transaction_key
    ));
}

#[tokio::test]
async fn ready_transaction_missing_chunk_returns_typed_error() {
    let (_chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-missing-ready-chunk", "0/16B7000"));
    let transaction_key = HeaderContext::from_message(&manifest_message)
        .expect("context")
        .transaction_key();
    let expected_partition_id = TransactionManifest::decode(manifest_message.payload.as_ref())
        .expect("manifest")
        .partitions[0]
        .id;
    let mut pending = HashMap::from([(
        transaction_key.clone(),
        PendingBarrierTransaction {
            manifest: Some(pending_manifest(manifest_message)),
            commit_marker: Some(pending_commit_marker(commit_message)),
            chunks: Default::default(),
        },
    )]);
    let mut consumer = RecordingConsumer::new(Vec::new());
    let mut applier = RecordingApplier::new(Vec::new());

    let error = crate::worker_transaction::apply_ready_transaction(
        &mut consumer,
        &mut applier,
        &mut pending,
        &transaction_key,
    )
    .await
    .expect_err("missing chunk");

    assert!(matches!(
        error,
        ApplyWorkerError::ReadyTransactionMissingChunk {
            transaction_id,
            partition_id,
        } if transaction_id == "tx-missing-ready-chunk" && partition_id == expected_partition_id
    ));
}
