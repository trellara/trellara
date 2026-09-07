use super::*;

#[test]
fn logical_frame_is_bounded_and_committed_as_one_transaction_record() {
    let mut builder = NativeLogicalFrameBuilder::default();
    builder
        .begin(42, 0x16B6C40, "source-a", "orders")
        .expect("begin frame");
    builder
        .append_change_start(NativeLogicalOperation::Insert, 16_384, b"public", b"orders")
        .expect("change header");
    builder.append_tuple_start(2).expect("tuple header");
    builder
        .append_column(23, NativeLogicalColumnStatus::Value, b"7")
        .expect("id column");
    builder
        .append_column(25, NativeLogicalColumnStatus::Null, &[])
        .expect("nullable column");
    builder.finish_event().expect("finish event");

    let frame = builder.finish(0x16B6C50).expect("commit frame");

    assert_eq!(&frame[..4], b"TRLD");
    assert!(frame.len() < MAX_LOGICAL_FRAME_BYTES);
    assert!(!builder.is_active());
}

#[test]
fn logical_frame_abort_discards_partial_transaction() {
    let mut builder = NativeLogicalFrameBuilder::default();
    builder
        .begin(42, 0x16B6C40, "source-a", "orders")
        .expect("begin frame");
    builder.abort();

    assert!(!builder.is_active());
    assert_eq!(
        builder.finish(0x16B6C50),
        Err(NativeLogicalFrameError::NotActive)
    );
}

#[test]
fn logical_frame_fails_closed_when_payload_exceeds_fixed_capacity() {
    let mut builder = NativeLogicalFrameBuilder::default();
    builder
        .begin(42, 0x16B6C40, "source-a", "orders")
        .expect("begin frame");
    builder
        .append_change_start(NativeLogicalOperation::Insert, 16_384, b"public", b"orders")
        .expect("change header");
    builder.append_tuple_start(1).expect("tuple header");

    assert!(matches!(
        builder.append_column(
            25,
            NativeLogicalColumnStatus::Value,
            &[7; MAX_LOGICAL_FRAME_BYTES]
        ),
        Err(NativeLogicalFrameError::FrameTooLarge { .. })
    ));
}

#[test]
fn logical_frame_can_mark_unchanged_toast_columns() {
    let mut builder = NativeLogicalFrameBuilder::default();
    builder
        .begin(42, 0x16B6C40, "source-a", "orders")
        .expect("begin frame");
    builder
        .append_change_start(NativeLogicalOperation::Update, 16_384, b"public", b"orders")
        .expect("change header");
    builder.append_tuple_presence(false).expect("old tuple");
    builder.append_tuple_presence(true).expect("new tuple");
    builder.append_tuple_start(1).expect("tuple header");
    builder
        .append_column(25, NativeLogicalColumnStatus::UnchangedToast, &[])
        .expect("unchanged toast marker");
    builder.finish_event().expect("finish event");

    let frame = builder.finish(0x16B6C50).expect("commit frame");
    let unchanged_toast_encoding = [
        0,
        0,
        0,
        25,
        NativeLogicalColumnStatus::UnchangedToast as u8,
        0,
        0,
        0,
        0,
    ];

    assert!(frame
        .windows(unchanged_toast_encoding.len())
        .any(|window| window == unchanged_toast_encoding));
}

#[test]
fn logical_frame_can_encode_logical_messages() {
    let mut builder = NativeLogicalFrameBuilder::default();
    builder
        .begin(0, 0x16B6C40, "source-a", "orders")
        .expect("begin frame");
    builder
        .append_message(0x16B6C41, false, b"trellara", b"checkpoint")
        .expect("message");
    builder.finish_event().expect("finish event");

    let frame = builder.finish(0x16B6C41).expect("commit frame");

    assert!(frame.contains(&(NativeLogicalOperation::Message as u8)));
    assert!(frame.ends_with(b"checkpoint"));
}
