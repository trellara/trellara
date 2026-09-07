use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use trellara_pg_extension::{NativeRelayWireAck, NativeRelayWireFrame};

use crate::native_spool::{NativeSpool, NativeSpoolWrite};
use crate::native_transport::{handle_connection, run_native_relay, NativeRelayServerConfig};

const SECRET: &[u8] = b"integration-test-secret";
static NEXT_TEST_PATH: AtomicU64 = AtomicU64::new(0);

#[test]
#[ignore = "manual live PostgreSQL harness entry point"]
fn live_server_from_environment() {
    let config = NativeRelayServerConfig {
        socket_path: environment_path("TRELLARA_NATIVE_RELAY_SOCKET"),
        spool_path: environment_path("TRELLARA_NATIVE_RELAY_SPOOL"),
        secret: std::env::var("TRELLARA_NATIVE_RELAY_SECRET")
            .expect("TRELLARA_NATIVE_RELAY_SECRET")
            .into_bytes(),
        exit_after_sync: std::env::var_os("TRELLARA_NATIVE_RELAY_EXIT_AFTER_SYNC").is_some(),
    };
    run_native_relay(config).expect("run native relay");
}

#[test]
fn durable_spool_acknowledges_and_deduplicates_exact_replay() {
    let paths = TestPaths::new();
    let config = paths.config();
    let frame = frame(0x16, b"committed transaction");

    let first_ack = exchange(&config, &frame);
    let first_size = fs::metadata(&paths.spool).expect("spool metadata").len();
    let replay_ack = exchange(&config, &frame);
    let replay_size = fs::metadata(&paths.spool).expect("spool metadata").len();

    assert_eq!(first_ack, replay_ack);
    assert_eq!(first_ack.commit_lsn, frame.commit_lsn);
    assert_eq!(first_ack.payload_digest, frame.payload_digest);
    assert_eq!(first_size, replay_size);
}

#[test]
fn conflicting_durable_evidence_at_the_same_lsn_fails_closed() {
    let paths = TestPaths::new();
    let first = frame(0x20, b"first");
    let conflicting = frame(0x20, b"different");
    let mut spool = NativeSpool::open(&paths.spool, SECRET).expect("open spool");
    let first_encoded = first.encode_authenticated(SECRET).expect("encode first");
    let conflicting_encoded = conflicting
        .encode_authenticated(SECRET)
        .expect("encode conflicting");

    assert_eq!(
        spool
            .persist(&first, &first_encoded)
            .expect("persist first"),
        NativeSpoolWrite::Appended
    );
    let error = spool
        .persist(&conflicting, &conflicting_encoded)
        .expect_err("conflict must fail");
    assert!(error.to_string().contains("conflicts with prior evidence"));
}

#[test]
fn restart_truncates_only_an_incomplete_tail_and_keeps_durable_proof() {
    let paths = TestPaths::new();
    let frame = frame(0x30, b"durable");
    let encoded = frame.encode_authenticated(SECRET).expect("encode");
    let durable_length = {
        let mut spool = NativeSpool::open(&paths.spool, SECRET).expect("open spool");
        spool.persist(&frame, &encoded).expect("persist");
        fs::metadata(&paths.spool).expect("metadata").len()
    };
    let mut file = OpenOptions::new()
        .append(true)
        .open(&paths.spool)
        .expect("append partial tail");
    file.write_all(&99_u32.to_be_bytes())
        .and_then(|()| file.write_all(b"partial"))
        .expect("write partial tail");
    file.sync_all().expect("sync partial tail");

    let mut recovered = NativeSpool::open(&paths.spool, SECRET).expect("recover spool");
    assert_eq!(
        fs::metadata(&paths.spool).expect("metadata").len(),
        durable_length
    );
    assert_eq!(
        recovered.persist(&frame, &encoded).expect("deduplicate"),
        NativeSpoolWrite::AlreadyDurable
    );
}

fn exchange(config: &NativeRelayServerConfig, frame: &NativeRelayWireFrame) -> NativeRelayWireAck {
    let (mut client, mut server) = UnixStream::pair().expect("socket pair");
    let config = config.clone();
    let server_thread = thread::spawn(move || {
        let mut spool = NativeSpool::open(&config.spool_path, &config.secret).expect("open spool");
        handle_connection(&mut server, &config, &mut spool).expect("handle frame");
    });
    let encoded = frame.encode_authenticated(SECRET).expect("encode frame");
    client
        .write_all(&u32::try_from(encoded.len()).expect("length").to_be_bytes())
        .and_then(|()| client.write_all(&encoded))
        .expect("write frame");
    let mut length = [0; 4];
    client.read_exact(&mut length).expect("read ACK length");
    let mut ack = vec![0; u32::from_be_bytes(length) as usize];
    client.read_exact(&mut ack).expect("read ACK");
    server_thread.join().expect("join relay");
    NativeRelayWireAck::decode_authenticated(&ack, SECRET).expect("authenticated ACK")
}

fn frame(commit_lsn: u64, payload: &[u8]) -> NativeRelayWireFrame {
    NativeRelayWireFrame::new(7, commit_lsn, "source", "dataset", payload.to_vec())
        .expect("valid frame")
}

fn environment_path(name: &str) -> PathBuf {
    std::env::var_os(name).map_or_else(|| panic!("{name}"), PathBuf::from)
}

struct TestPaths {
    root: PathBuf,
    spool: PathBuf,
}

impl TestPaths {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "trellara-native-relay-{}-{unique}-{}",
            std::process::id(),
            NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("create test directory");
        let spool = root.join("relay.spool");
        Self { root, spool }
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
