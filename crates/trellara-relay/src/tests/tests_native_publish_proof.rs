use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use trellara_pg_extension::NativeRelayWireFrame;

use crate::native_publish_ledger::NativePublishLedger;
use crate::native_publish_proof::{NativeKafkaPublishProof, NativePublishDestination};
use crate::native_transport::NativeRelayTransportError;

static NEXT_TEST_PATH: AtomicU64 = AtomicU64::new(0);

#[test]
fn native_kafka_publish_proof_round_trips_exact_destination_and_digest() {
    let frame = frame(0x16_b6c50, b"committed transaction");
    let proof = NativeKafkaPublishProof::new(
        &frame,
        destination("cluster-a", "trellara.source.sales"),
        42,
    )
    .expect("proof");
    let encoded = proof.encode().expect("encode proof");

    let decoded = NativeKafkaPublishProof::decode(&encoded).expect("decode proof");

    assert_eq!(decoded, proof);
    assert_eq!(decoded.payload_digest, frame.payload_digest);
    assert_eq!(decoded.destination.partition, 0);
    assert_eq!(decoded.offset, 42);
}

#[test]
fn native_kafka_publish_proof_rejects_tampered_bytes() {
    let frame = frame(0x16_b6c50, b"committed transaction");
    let proof = NativeKafkaPublishProof::new(
        &frame,
        destination("cluster-a", "trellara.source.sales"),
        42,
    )
    .expect("proof");
    let mut encoded = proof.encode().expect("encode proof");
    let byte = encoded
        .last_mut()
        .expect("proof includes checksum byte to tamper");
    *byte ^= 0xff;

    assert!(matches!(
        NativeKafkaPublishProof::decode(&encoded),
        Err(NativeRelayTransportError::PublishProofCorrupt(
            "publish proof checksum mismatch"
        ))
    ));
}

#[test]
fn native_publish_ledger_deduplicates_exact_proof_after_restart() {
    let path = test_path("publish-ledger");
    let frame = frame(0x20, b"durable publish");
    let destination = destination("cluster-a", "trellara.source.sales");
    let proof =
        NativeKafkaPublishProof::new(&frame, destination.clone(), 7).expect("publish proof");

    {
        let mut ledger = NativePublishLedger::open(&path).expect("open ledger");
        assert!(!ledger.proven(&frame, &destination).expect("empty ledger"));
        ledger.persist(proof.clone()).expect("persist proof");
        assert!(ledger.proven(&frame, &destination).expect("proof durable"));
    }

    let mut recovered = NativePublishLedger::open(&path).expect("recover ledger");
    assert!(recovered
        .proven(&frame, &destination)
        .expect("proof durable after restart"));
    recovered.persist(proof).expect("idempotent replay");
    let _ = fs::remove_file(path);
}

#[test]
fn native_publish_ledger_fails_closed_on_conflicting_destination() {
    let path = test_path("publish-ledger-conflict");
    let frame = frame(0x30, b"durable publish");
    let mut ledger = NativePublishLedger::open(&path).expect("open ledger");
    let proof =
        NativeKafkaPublishProof::new(&frame, destination("cluster-a", "trellara.source.sales"), 7)
            .expect("publish proof");
    ledger.persist(proof).expect("persist proof");

    let error = ledger
        .proven(&frame, &destination("cluster-b", "trellara.source.sales"))
        .expect_err("different destination conflicts");

    assert!(matches!(
        error,
        NativeRelayTransportError::ConflictingKafkaProof { commit_lsn: 0x30 }
    ));
    let _ = fs::remove_file(path);
}

#[test]
fn native_publish_ledger_truncates_incomplete_tail_on_restart() {
    let path = test_path("publish-ledger-tail");
    let frame = frame(0x40, b"durable publish");
    let destination = destination("cluster-a", "trellara.source.sales");
    let proof = NativeKafkaPublishProof::new(&frame, destination.clone(), 7).expect("proof");
    let durable_len = {
        let mut ledger = NativePublishLedger::open(&path).expect("open ledger");
        ledger.persist(proof).expect("persist proof");
        fs::metadata(&path).expect("metadata").len()
    };
    let mut file = OpenOptions::new()
        .append(true)
        .open(&path)
        .expect("append tail");
    file.write_all(&99_u32.to_be_bytes())
        .and_then(|()| file.write_all(b"partial"))
        .expect("write incomplete tail");
    file.sync_all().expect("sync tail");

    let recovered = NativePublishLedger::open(&path).expect("recover ledger");

    assert_eq!(fs::metadata(&path).expect("metadata").len(), durable_len);
    assert!(recovered
        .proven(&frame, &destination)
        .expect("proof survives tail truncation"));
    let _ = fs::remove_file(path);
}

fn frame(commit_lsn: u64, payload: &[u8]) -> NativeRelayWireFrame {
    NativeRelayWireFrame::new(7, commit_lsn, "source", "sales", payload.to_vec()).expect("frame")
}

fn destination(cluster_id: &str, topic: &str) -> NativePublishDestination {
    NativePublishDestination {
        cluster_id: cluster_id.to_string(),
        topic: topic.to_string(),
        partition: 0,
    }
}

fn test_path(name: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "trellara-{name}-{}-{unique}-{}",
        std::process::id(),
        NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed)
    ))
}
