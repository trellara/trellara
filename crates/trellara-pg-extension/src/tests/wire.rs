use super::*;

#[test]
fn relay_wire_round_trip_authenticates_complete_frame_and_ack() {
    let secret = b"rotated-test-secret";
    let frame = NativeRelayWireFrame::new(
        42,
        0x16B6C50,
        "source-a",
        "orders",
        b"committed-frame".to_vec(),
    )
    .expect("wire frame");
    let encoded = frame.encode_authenticated(secret).expect("encode frame");

    let decoded =
        NativeRelayWireFrame::decode_authenticated(&encoded, secret).expect("decode frame");
    assert_eq!(decoded, frame);

    let ack = native_relay_ack(&decoded);
    let encoded_ack = ack.encode_authenticated(secret).expect("encode ack");
    assert_eq!(
        NativeRelayWireAck::decode_authenticated(&encoded_ack, secret).expect("decode ack"),
        ack
    );
}

#[test]
fn relay_wire_rejects_tampering_and_wrong_secret() {
    let frame = NativeRelayWireFrame::new(
        42,
        0x16B6C50,
        "source-a",
        "orders",
        b"committed-frame".to_vec(),
    )
    .expect("wire frame");
    let mut encoded = frame
        .encode_authenticated(b"correct-secret")
        .expect("encode frame");
    encoded[30] ^= 0x01;

    assert!(matches!(
        NativeRelayWireFrame::decode_authenticated(&encoded, b"correct-secret"),
        Err(NativeRelayWireError::AuthenticationFailed)
    ));
    let untouched = frame
        .encode_authenticated(b"correct-secret")
        .expect("encode frame");
    assert_eq!(
        NativeRelayWireFrame::decode_authenticated(&untouched, b"wrong-secret"),
        Err(NativeRelayWireError::AuthenticationFailed)
    );
}
