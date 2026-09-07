use super::*;
use crate::barrier::{
    HeaderContext, PendingBarrierTransaction, PendingChunk, PendingCommitMarker, PendingManifest,
};
use std::collections::HashMap;

#[path = "ready_state_errors/cases.rs"]
mod ready_state_cases;

fn pending_manifest(message: StreamMessage) -> PendingManifest {
    PendingManifest {
        manifest: TransactionManifest::decode(message.payload.as_ref()).expect("manifest"),
        context: HeaderContext::from_message(&message).expect("context"),
        messages: vec![message],
    }
}

fn pending_commit_marker(message: StreamMessage) -> PendingCommitMarker {
    PendingCommitMarker {
        marker: TransactionCommitMarker::decode(message.payload.as_ref()).expect("commit marker"),
        messages: vec![message],
    }
}

fn pending_chunk(message: StreamMessage) -> PendingChunk {
    PendingChunk {
        chunk: PartitionChunk::decode(message.payload.as_ref()).expect("chunk"),
        messages: vec![message],
    }
}
