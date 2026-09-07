use super::*;
use trellara_pg_extension::{
    native_committed_transaction_frame, native_handoff_drain_batch, native_handoff_queue_config,
    native_hook_registration_plan, native_relay_auth_plan, native_shared_memory_lifecycle_gate,
    native_shared_memory_plan, NativeSharedMemoryPhase, NativeWorkerSupervisionState,
};
use trellara_protocol::{
    ChangeRecord, ColumnValue, Operation, RelationId, ReplicaIdentity, RowImage, StrictEnvelope,
    TransactionEnvelope,
};
use trellara_stream::StreamMessage;

#[test]
fn native_worker_runner_produces_feedback_after_supervised_drain() {
    let step = relay_step("0/16B6C50");
    let batch = native_batch(step.envelope.checksum, "0/16B6C50");
    let state = worker_state(true);

    let outcome = run_native_worker_once(&state, &batch, &step).expect("worker run");

    assert!(matches!(
        outcome,
        NativeWorkerRunOutcome::SourceFeedbackReady(decision)
            if decision.source_feedback_lsn == "0/16B6C50" && decision.frames_covered == 1
    ));
}

#[test]
fn native_worker_runner_sleeps_without_frames() {
    let step = relay_step("0/16B6C50");
    let batch = native_batch(step.envelope.checksum, "0/16B6C50");
    let state = worker_state(false);

    assert_eq!(
        run_native_worker_once(&state, &batch, &step).expect("worker run"),
        NativeWorkerRunOutcome::Slept
    );
}

#[test]
fn native_worker_runner_fails_closed_before_supervision_is_ready() {
    let step = relay_step("0/16B6C50");
    let batch = native_batch(step.envelope.checksum, "0/16B6C50");
    let mut state = worker_state(true);
    state.relay_auth_plan = None;

    assert!(matches!(
        run_native_worker_once(&state, &batch, &step),
        Err(RelayError::NativeWorkerSupervisionRejected(
            trellara_pg_extension::NativeWorkerSupervisionError::RelayNotConfigured
        ))
    ));
}

#[test]
fn native_worker_report_explains_success_sleep_and_fail_closed() {
    let step = relay_step("0/16B6C50");
    let batch = native_batch(step.envelope.checksum, "0/16B6C50");

    let success = run_native_worker_once_report(&worker_state(true), &batch, &step);
    assert_eq!(success.status, NativeWorkerRunStatus::SourceFeedbackReady);
    assert_eq!(success.drained_frames, 1);
    assert_eq!(success.source_feedback_lsn, Some("0/16B6C50".to_string()));
    assert_eq!(success.reason, "durable_relay_proof_allows_source_feedback");

    let slept = run_native_worker_once_report(&worker_state(false), &batch, &step);
    assert_eq!(slept.status, NativeWorkerRunStatus::Slept);
    assert_eq!(slept.reason, "queue_empty_after_supervision_ready");

    let mut blocked = worker_state(true);
    blocked.relay_auth_plan = None;
    let failed = run_native_worker_once_report(&blocked, &batch, &step);
    assert_eq!(failed.status, NativeWorkerRunStatus::FailedClosed);
    assert_eq!(failed.reason, "supervision_rejected");
}

fn worker_state(queue_has_frames: bool) -> NativeWorkerSupervisionState {
    let config = native_handoff_queue_config(8, 2048, 8192).expect("queue config");
    let plan = native_shared_memory_plan(&config).expect("shared memory plan");
    NativeWorkerSupervisionState {
        hook_plan: native_hook_registration_plan(true, true, true),
        shared_memory_gate: native_shared_memory_lifecycle_gate(
            &plan,
            NativeSharedMemoryPhase::CaptureStart,
            true,
            true,
        )
        .expect("shared memory lifecycle gate"),
        relay_auth_plan: Some(
            native_relay_auth_plan(
                "trellara-relay@cluster-a",
                "pg_parameter:trellara.relay_secret",
                "/var/run/postgresql/trellara-relay.sock",
            )
            .expect("relay auth plan"),
        ),
        queue_has_frames,
    }
}

fn native_batch(checksum: u64, commit_lsn: &str) -> trellara_pg_extension::NativeHandoffDrainBatch {
    let config = native_handoff_queue_config(8, 2048, 8192).expect("queue config");
    let frame = native_committed_transaction_frame(
        "source",
        "sales",
        "tx-native-worker",
        commit_lsn,
        1024,
        checksum,
    )
    .expect("native handoff frame");
    native_handoff_drain_batch(&config, vec![frame]).expect("native handoff batch")
}

fn relay_step(source_ack_lsn: &str) -> RelayStep {
    let envelope = envelope();
    let messages = vec![StreamMessage::strict_transaction(&envelope).expect("stream message")];
    let publish_acks = vec![PublishAck {
        topic: "trellara.source.sales.strict".to_string(),
        partition: 0,
        offset: 7,
    }];
    let source_ack_boundary = SourceAckBoundaryProof::recorded(
        &envelope,
        &messages,
        &publish_acks,
        messages.len(),
        source_ack_lsn,
    )
    .expect("source ack proof");

    RelayStep {
        envelope,
        published_messages: messages,
        publish_acks,
        source_ack_lsn: source_ack_lsn.to_string(),
        source_ack_boundary,
    }
}

fn envelope() -> TransactionEnvelope {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-native-worker".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_700_000_000,
        changes: vec![ChangeRecord {
            transaction_id: "tx-native-worker".to_string(),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            idempotency_key: "source:0/16B6C50:tx-native-worker:1".to_string(),
            relation: Some(RelationId::new(42, "public", "sales")),
            after: Some(RowImage::new(vec![ColumnValue::text(
                "store_id", 25, "store-1", true,
            )])),
        }],
    });
    envelope.finalize_checksum();
    envelope
}
