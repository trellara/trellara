use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use trellara_pg_extension::{native_relay_ack, NativeRelayWireFrame};

#[cfg(feature = "kafka")]
use crate::native_kafka::{NativeFramePublisher, NativeKafkaPublisher, NativeKafkaRelayConfig};
#[cfg(feature = "kafka")]
use crate::native_publish_ledger::NativePublishLedger;
#[cfg(feature = "kafka")]
use crate::native_publish_proof::frame_key;
use crate::native_spool::NativeSpool;
pub use crate::native_transport_error::{NativeRelayTransportError, NativeRelayTransportResult};

#[path = "native_transport/socket_io.rs"]
mod socket_io;

use socket_io::{read_message, write_message};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeRelayServerConfig {
    pub socket_path: PathBuf,
    pub spool_path: PathBuf,
    pub secret: Vec<u8>,
    pub exit_after_sync: bool,
}

pub fn run_native_relay(config: NativeRelayServerConfig) -> NativeRelayTransportResult<()> {
    let listener = bind_listener(&config)?;
    let mut spool = NativeSpool::open(&config.spool_path, &config.secret)?;
    for connection in listener.incoming() {
        let mut stream = connection?;
        handle_connection(&mut stream, &config, &mut spool)?;
    }
    Ok(())
}

#[cfg(feature = "kafka")]
pub fn run_native_kafka_relay(
    config: NativeRelayServerConfig,
    kafka: NativeKafkaRelayConfig,
) -> NativeRelayTransportResult<()> {
    let publisher = NativeKafkaPublisher::new(&kafka)?;
    let listener = bind_listener(&config)?;
    let mut spool = NativeSpool::open(&config.spool_path, &config.secret)?;
    let mut ledger = NativePublishLedger::open(&kafka.proof_path)?;
    for connection in listener.incoming() {
        let mut stream = connection?;
        handle_kafka_connection(&mut stream, &config, &mut spool, &mut ledger, &publisher)?;
    }
    Ok(())
}

pub(crate) fn handle_connection(
    stream: &mut UnixStream,
    config: &NativeRelayServerConfig,
    spool: &mut NativeSpool,
) -> NativeRelayTransportResult<()> {
    let encoded = read_message(stream)?;
    let frame = NativeRelayWireFrame::decode_authenticated(&encoded, &config.secret)?;
    spool.persist(&frame, &encoded)?;
    if config.exit_after_sync {
        std::process::exit(86);
    }
    let ack = native_relay_ack(&frame).encode_authenticated(&config.secret)?;
    write_message(stream, &ack)
}

#[cfg(feature = "kafka")]
pub(crate) fn handle_kafka_connection(
    stream: &mut UnixStream,
    config: &NativeRelayServerConfig,
    spool: &mut NativeSpool,
    ledger: &mut NativePublishLedger,
    publisher: &dyn NativeFramePublisher,
) -> NativeRelayTransportResult<()> {
    let encoded = read_message(stream)?;
    let frame = NativeRelayWireFrame::decode_authenticated(&encoded, &config.secret)?;
    spool.persist(&frame, &encoded)?;
    let destination = publisher.destination(&frame)?;
    if !ledger.proven(&frame, &destination)? {
        let proof = publisher.publish(&frame)?;
        if proof.key != frame_key(&frame)
            || proof.payload_digest != frame.payload_digest
            || proof.destination != destination
        {
            return Err(NativeRelayTransportError::KafkaProofMismatch);
        }
        ledger.persist(proof)?;
    }
    if config.exit_after_sync {
        std::process::exit(86);
    }
    let ack = native_relay_ack(&frame).encode_authenticated(&config.secret)?;
    write_message(stream, &ack)
}

fn prepare_socket(path: &Path) -> NativeRelayTransportResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if !metadata.file_type().is_socket() {
        return Err(NativeRelayTransportError::UnsafeSocketPath(
            path.to_path_buf(),
        ));
    }
    fs::remove_file(path)?;
    Ok(())
}

fn bind_listener(config: &NativeRelayServerConfig) -> NativeRelayTransportResult<UnixListener> {
    if config.secret.is_empty() {
        return Err(NativeRelayTransportError::EmptySecret);
    }
    prepare_socket(&config.socket_path)?;
    let listener = UnixListener::bind(&config.socket_path)?;
    fs::set_permissions(&config.socket_path, fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}
