use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use trellara_pg_extension::{NativeRelayWireAck, NativeRelayWireFrame};

use crate::native_kafka::NativeFramePublisher;
use crate::native_publish_ledger::NativePublishLedger;
use crate::native_publish_proof::{NativeKafkaPublishProof, NativePublishDestination};
use crate::native_spool::NativeSpool;
use crate::native_transport::{
    handle_kafka_connection, NativeRelayServerConfig, NativeRelayTransportError,
    NativeRelayTransportResult,
};

const SECRET: &[u8] = b"native-kafka-test-secret";
static TEST_PATH_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn fsynced_kafka_proof_precedes_ack_and_skips_exact_replay() {
    let paths = TestPaths::new();
    let publisher = RecordingPublisher::succeeding();
    let frame = test_frame();

    let first = exchange(&paths, &publisher, &frame).expect("first ACK");
    let proof_size = fs::metadata(&paths.proof).expect("proof metadata").len();
    let replay = exchange(&paths, &publisher, &frame).expect("replay ACK");

    assert_eq!(first, replay);
    assert_eq!(publisher.calls(), 1);
    assert_eq!(
        fs::metadata(&paths.proof).expect("proof metadata").len(),
        proof_size
    );
}

#[test]
fn ambiguous_publish_is_not_acknowledged_and_is_replayed() {
    let paths = TestPaths::new();
    let publisher = RecordingPublisher::failing();
    let frame = test_frame();

    let error = exchange(&paths, &publisher, &frame).expect_err("missing broker proof");
    assert!(matches!(error, NativeRelayTransportError::KafkaPublish(_)));
    assert_eq!(publisher.calls(), 1);

    publisher.allow_success();
    let ack = exchange(&paths, &publisher, &frame).expect("replayed ACK");
    assert_eq!(ack.commit_lsn, frame.commit_lsn);
    assert_eq!(publisher.calls(), 2);
}

fn exchange(
    paths: &TestPaths,
    publisher: &RecordingPublisher,
    frame: &NativeRelayWireFrame,
) -> NativeRelayTransportResult<NativeRelayWireAck> {
    let (mut client, mut server) = UnixStream::pair()?;
    let config = paths.config();
    let publisher = publisher.clone();
    let spool_path = paths.spool.clone();
    let proof_path = paths.proof.clone();
    let server_thread = thread::spawn(move || {
        let mut spool = NativeSpool::open(&spool_path, SECRET)?;
        let mut ledger = NativePublishLedger::open(&proof_path)?;
        handle_kafka_connection(&mut server, &config, &mut spool, &mut ledger, &publisher)
    });
    let encoded = frame.encode_authenticated(SECRET)?;
    client.write_all(&u32::try_from(encoded.len())?.to_be_bytes())?;
    client.write_all(&encoded)?;
    let mut length = [0; 4];
    let read_result = client.read_exact(&mut length);
    let server_result = server_thread
        .join()
        .map_err(|_| NativeRelayTransportError::KafkaPublish("test server panicked".to_string()))?;
    server_result?;
    read_result?;
    let mut ack = vec![0; u32::from_be_bytes(length) as usize];
    client.read_exact(&mut ack)?;
    Ok(NativeRelayWireAck::decode_authenticated(&ack, SECRET)?)
}

#[derive(Clone)]
struct RecordingPublisher {
    calls: Arc<AtomicUsize>,
    fail: Arc<AtomicBool>,
}

impl RecordingPublisher {
    fn succeeding() -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            fail: Arc::new(AtomicBool::new(false)),
        }
    }

    fn failing() -> Self {
        let publisher = Self::succeeding();
        publisher.fail.store(true, Ordering::SeqCst);
        publisher
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn allow_success(&self) {
        self.fail.store(false, Ordering::SeqCst);
    }
}

impl NativeFramePublisher for RecordingPublisher {
    fn destination(
        &self,
        _frame: &NativeRelayWireFrame,
    ) -> NativeRelayTransportResult<NativePublishDestination> {
        Ok(NativePublishDestination {
            cluster_id: "test-cluster".to_string(),
            topic: "trellara.source.dataset.strict".to_string(),
            partition: 0,
        })
    }

    fn publish(
        &self,
        frame: &NativeRelayWireFrame,
    ) -> NativeRelayTransportResult<NativeKafkaPublishProof> {
        let offset = self.calls.fetch_add(1, Ordering::SeqCst) as i64;
        if self.fail.load(Ordering::SeqCst) {
            return Err(NativeRelayTransportError::KafkaPublish(
                "ambiguous broker outcome".to_string(),
            ));
        }
        NativeKafkaPublishProof::new(frame, self.destination(frame)?, offset)
    }
}

fn test_frame() -> NativeRelayWireFrame {
    NativeRelayWireFrame::new(7, 0x16_b6c50, "source", "dataset", vec![1, 2, 3]).expect("frame")
}

struct TestPaths {
    root: PathBuf,
    spool: PathBuf,
    proof: PathBuf,
}

impl TestPaths {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let id = TEST_PATH_ID.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!("trellara-kafka-proof-{unique}-{id}"));
        fs::create_dir_all(&root).expect("test directory");
        Self {
            spool: root.join("relay.spool"),
            proof: root.join("relay.kafka-proof"),
            root,
        }
    }

    fn config(&self) -> NativeRelayServerConfig {
        NativeRelayServerConfig {
            socket_path: self.root.join("relay.sock"),
            spool_path: self.spool.clone(),
            secret: SECRET.to_vec(),
            exit_after_sync: false,
        }
    }
}

impl Drop for TestPaths {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
