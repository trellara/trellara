use super::super::*;

#[test]
fn ready_envelope_reconstruction_preserves_manifest_transaction_boundary() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-ready-envelope", "0/16B7000"));
    let transaction_key = HeaderContext::from_message(&manifest_message)
        .expect("context")
        .transaction_key();
    let manifest =
        TransactionManifest::decode(manifest_message.payload.as_ref()).expect("manifest");
    let chunk = PartitionChunk::decode(chunk_message.payload.as_ref()).expect("chunk");
    let partition_id = chunk.partition_id;
    let pending = PendingBarrierTransaction {
        manifest: Some(pending_manifest(manifest_message)),
        commit_marker: Some(pending_commit_marker(commit_message)),
        chunks: HashMap::from([(partition_id, pending_chunk(chunk_message))])
            .into_iter()
            .collect(),
    };

    let envelope = crate::worker_ready_envelope::build_ready_envelope(&pending, &transaction_key)
        .expect("ready envelope");

    assert_eq!(envelope.transaction_id, "tx-ready-envelope");
    assert_eq!(envelope.commit_lsn, "0/16B7000");
    assert_eq!(envelope.source_id, "source");
    assert_eq!(envelope.dataset_id, "sales");
    assert_eq!(envelope.changes.len(), manifest.global_event_count as usize);
    assert_eq!(
        envelope.manifest.as_ref().map(|manifest| (
            manifest.transaction_id.as_str(),
            manifest.source_commit_lsn.as_str()
        )),
        Some(("tx-ready-envelope", "0/16B7000"))
    );
}

#[test]
fn ready_envelope_reconstruction_rejects_unlisted_buffered_chunk() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-ready-extra-chunk", "0/16B7000"));
    let transaction_key = HeaderContext::from_message(&manifest_message)
        .expect("context")
        .transaction_key();
    let chunk = PartitionChunk::decode(chunk_message.payload.as_ref()).expect("chunk");
    let partition_id = chunk.partition_id;
    let mut unlisted_chunk = chunk.clone();
    unlisted_chunk.partition_id = 999;
    let pending = PendingBarrierTransaction {
        manifest: Some(pending_manifest(manifest_message)),
        commit_marker: Some(pending_commit_marker(commit_message)),
        chunks: HashMap::from([
            (
                partition_id,
                PendingChunk {
                    chunk,
                    messages: vec![chunk_message.clone()],
                },
            ),
            (
                999,
                PendingChunk {
                    chunk: unlisted_chunk,
                    messages: vec![chunk_message],
                },
            ),
        ])
        .into_iter()
        .collect(),
    };

    let error = crate::worker_ready_envelope::build_ready_envelope(&pending, &transaction_key)
        .expect_err("unlisted chunk rejected");

    assert!(matches!(
        error,
        ApplyWorkerError::Protocol(ProtocolError::PartitionNotInManifest {
            transaction_id,
            partition_id: 999,
        }) if transaction_id == "tx-ready-extra-chunk"
    ));
}

#[test]
fn ready_envelope_reconstruction_revalidates_header_identity() {
    let (chunk_message, manifest_message, commit_message) =
        partitioned_messages(&envelope("tx-ready-spaced-source", "0/16B7000"));
    let transaction_key = HeaderContext::from_message(&manifest_message)
        .expect("context")
        .transaction_key();
    let chunk = PartitionChunk::decode(chunk_message.payload.as_ref()).expect("chunk");
    let partition_id = chunk.partition_id;
    let mut manifest = pending_manifest(manifest_message);
    manifest.context.source_id = " source ".to_string();
    let pending = PendingBarrierTransaction {
        manifest: Some(manifest),
        commit_marker: Some(pending_commit_marker(commit_message)),
        chunks: HashMap::from([(partition_id, pending_chunk(chunk_message))])
            .into_iter()
            .collect(),
    };

    let error = crate::worker_ready_envelope::build_ready_envelope(&pending, &transaction_key)
        .expect_err("invalid reconstructed envelope");

    assert!(matches!(
        error,
        ApplyWorkerError::Protocol(ProtocolError::InvalidEnvelopeField {
            field: "source_id",
            ..
        })
    ));
}
