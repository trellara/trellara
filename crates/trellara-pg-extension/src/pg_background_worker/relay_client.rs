use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use crate::{NativeRelayWireAck, NativeRelayWireFrame};

const MAX_ACK_BYTES: usize = 1024;

pub(super) fn exchange_with_relay(
    frame: &NativeRelayWireFrame,
    secret: &[u8],
) -> Result<NativeRelayWireAck, String> {
    let encoded = frame
        .encode_authenticated(secret)
        .map_err(|error| error.to_string())?;
    let length = u32::try_from(encoded.len())
        .map_err(|_| "native relay frame length exceeds u32".to_string())?;
    let timeout = Duration::from_millis(crate::pg_guc::relay_timeout_milliseconds());
    let mut stream = UnixStream::connect(crate::pg_guc::relay_socket_path())
        .map_err(|error| format!("relay socket connect failed: {error}"))?;
    stream
        .set_read_timeout(Some(timeout))
        .and_then(|()| stream.set_write_timeout(Some(timeout)))
        .map_err(|error| format!("relay socket timeout setup failed: {error}"))?;
    stream
        .write_all(&length.to_be_bytes())
        .and_then(|()| stream.write_all(&encoded))
        .map_err(|error| format!("relay socket write failed: {error}"))?;

    read_relay_ack(&mut stream, secret)
}

fn read_relay_ack(stream: &mut UnixStream, secret: &[u8]) -> Result<NativeRelayWireAck, String> {
    let mut ack_length = [0; 4];
    stream
        .read_exact(&mut ack_length)
        .map_err(|error| format!("relay acknowledgement length read failed: {error}"))?;
    let ack_length = usize::try_from(u32::from_be_bytes(ack_length))
        .map_err(|_| "relay acknowledgement length is invalid".to_string())?;
    if ack_length == 0 || ack_length > MAX_ACK_BYTES {
        return Err(format!(
            "relay acknowledgement length {ack_length} exceeds {MAX_ACK_BYTES} bytes"
        ));
    }
    let mut encoded_ack = vec![0; ack_length];
    stream
        .read_exact(&mut encoded_ack)
        .map_err(|error| format!("relay acknowledgement read failed: {error}"))?;
    NativeRelayWireAck::decode_authenticated(&encoded_ack, secret)
        .map_err(|error| error.to_string())
}
