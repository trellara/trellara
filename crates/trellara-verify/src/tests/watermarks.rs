use super::*;

#[test]
fn lsn_parsing_orders_wal_positions() {
    assert!(parse_lsn("0/16B6C51") > parse_lsn("0/16B6C50"));
    assert!(parse_lsn("1/00000000") > parse_lsn("0/FFFFFFFF"));
}

#[test]
fn watermark_check_fails_when_target_is_behind() {
    assert!(ensure_target_caught_up("0/16B6C50", "0/16B6C50").is_ok());
    assert!(matches!(
        ensure_target_caught_up("0/16B6C50", "0/16B6B00"),
        Err(VerifyError::TargetBehind { .. })
    ));
}

#[test]
fn watermark_check_rejects_malformed_lsn_evidence() {
    assert!(matches!(
        ensure_target_caught_up("not-a-lsn", "0/16B6C50"),
        Err(VerifyError::InvalidWatermarkLsn { field, .. }) if field == "source"
    ));
    assert!(matches!(
        ensure_target_caught_up("0/16B6C50", "1/100000000"),
        Err(VerifyError::InvalidWatermarkLsn { field, .. }) if field == "target"
    ));
}
