use super::primitives::{put_i64, put_u64};
use crate::lsn::{format_lsn, parse_lsn};
use crate::replication::{
    parse_replication_copy_data, standby_status_update_payload, validate_copy_both_response,
    ReplicationCopyData,
};
use crate::replication_feedback::{standby_status_boundary_payload, StandbyStatusBoundary};
use crate::PgOutputReader;

#[test]
fn lsn_parser_round_trips_postgres_hex_format() {
    let lsn = parse_lsn("16/B6C50").expect("valid lsn");

    assert_eq!(lsn, 0x16_000B_6C50);
    assert_eq!(format_lsn(lsn), "16/B6C50");
    assert!(parse_lsn("not-an-lsn").is_err());
    assert!(parse_lsn("1/100000000").is_err());
}

#[test]
fn pgoutput_reader_reads_lsn_as_unsigned_wal_position() {
    let mut payload = Vec::new();
    put_u64(&mut payload, 0x8000_0000_0000_0000);
    let mut reader = PgOutputReader::new(&payload);

    assert_eq!(reader.read_lsn().expect("lsn"), "80000000/0");
    reader.expect_finished().expect("finished");
}

#[test]
fn replication_copy_data_parses_xlog_and_keepalive_boundaries() {
    let mut xlog = vec![b'w'];
    put_u64(&mut xlog, 0x16B6C00);
    put_u64(&mut xlog, 0x16B6C50);
    put_i64(&mut xlog, 123_000);
    xlog.extend_from_slice(b"pgoutput");

    match parse_replication_copy_data(&xlog).expect("xlog data") {
        ReplicationCopyData::XLogData(data) => {
            assert_eq!(data.wal_start, "0/16B6C00");
            assert_eq!(data.wal_end, "0/16B6C50");
            assert_eq!(data.send_timestamp_ms, 946_684_800_123);
            assert_eq!(data.payload, b"pgoutput");
        }
        ReplicationCopyData::PrimaryKeepalive(_) => panic!("expected xlog data"),
    }

    let mut keepalive = vec![b'k'];
    put_u64(&mut keepalive, 0x16B6C80);
    put_i64(&mut keepalive, 456_000);
    keepalive.push(1);

    match parse_replication_copy_data(&keepalive).expect("keepalive") {
        ReplicationCopyData::PrimaryKeepalive(keepalive) => {
            assert_eq!(keepalive.wal_end, "0/16B6C80");
            assert_eq!(keepalive.send_timestamp_ms, 946_684_800_456);
            assert!(keepalive.reply_requested);
        }
        ReplicationCopyData::XLogData(_) => panic!("expected keepalive"),
    }
}

#[test]
fn replication_copy_data_names_unknown_tag_by_raw_byte() {
    let error = match parse_replication_copy_data(&[0x01]) {
        Ok(_) => panic!("unknown CopyData tag should fail closed"),
        Err(error) => error,
    };

    assert!(
        matches!(error, crate::CaptureError::ReplicationProtocol(message) if message.contains("unsupported replication CopyData tag 0x01"))
    );
}

#[test]
fn copy_both_response_accepts_expected_column_formats() {
    let response = [0_u8, 0, 2, 0, 0, 0, 0];

    validate_copy_both_response(&response).expect("copy both response");
}

#[test]
fn copy_both_response_accepts_no_column_formats() {
    let response = [0_u8, 0, 0];

    validate_copy_both_response(&response).expect("copy both response");
}

#[test]
fn copy_both_response_rejects_absurd_column_count_before_reading_formats() {
    let response = [0_u8, 0xff, 0xff, 0, 0, 0, 0];

    let error = validate_copy_both_response(&response).expect_err("copy both response");

    assert!(
        matches!(error, crate::CaptureError::ReplicationProtocol(message) if message.contains("column count 65535"))
    );
}

#[test]
fn standby_status_update_payload_reports_acknowledged_lsn_for_every_watermark() {
    let payload = standby_status_update_payload(0x16B6C50, 789_000, false);

    assert_eq!(payload.len(), 34);
    let mut reader = PgOutputReader::new(&payload);
    assert_eq!(reader.read_u8().expect("tag"), b'r');
    assert_eq!(reader.read_lsn().expect("written lsn"), "0/16B6C50");
    assert_eq!(reader.read_lsn().expect("flushed lsn"), "0/16B6C50");
    assert_eq!(reader.read_lsn().expect("applied lsn"), "0/16B6C50");
    assert_eq!(reader.read_i64().expect("timestamp"), 789_000);
    assert_eq!(reader.read_u8().expect("reply flag"), 0);
    reader.expect_finished().expect("finished");
}

#[test]
fn standby_status_boundary_uses_only_last_durable_acknowledged_lsn() {
    let boundary = StandbyStatusBoundary::acknowledged(0x16B6C50, 789_000, true);

    assert!(boundary.uses_only_acknowledged_lsn());
    assert_eq!(boundary.written_lsn, 0x16B6C50);
    assert_eq!(boundary.flushed_lsn, 0x16B6C50);
    assert_eq!(boundary.applied_lsn, 0x16B6C50);

    let payload = standby_status_boundary_payload(&boundary);
    let mut reader = PgOutputReader::new(&payload);
    assert_eq!(reader.read_u8().expect("tag"), b'r');
    assert_eq!(reader.read_lsn().expect("written lsn"), "0/16B6C50");
    assert_eq!(reader.read_lsn().expect("flushed lsn"), "0/16B6C50");
    assert_eq!(reader.read_lsn().expect("applied lsn"), "0/16B6C50");
    assert_eq!(reader.read_i64().expect("timestamp"), 789_000);
    assert_eq!(reader.read_u8().expect("reply flag"), 1);
    reader.expect_finished().expect("finished");
}
