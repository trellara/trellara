use super::*;

#[test]
fn lsn_parser_orders_postgres_boundaries() {
    assert_eq!(parse_lsn("0/0").expect("zero lsn"), 0);
    assert_eq!(parse_lsn("0/16B6C50").expect("lsn"), 0x016B_6C50);
    assert!(parse_lsn("0/16B6C51").expect("lsn") > parse_lsn("0/16B6C50").expect("lsn"));
    assert!(parse_lsn("1/0").expect("lsn") > parse_lsn("0/FFFFFFFF").expect("lsn"));
    assert_eq!(format_lsn(0x1_0000_0000), "1/0");
}

#[test]
fn lsn_parser_rejects_invalid_boundaries() {
    assert!(matches!(
        parse_lsn("not-an-lsn"),
        Err(ProtocolError::InvalidLsn { .. })
    ));
    assert!(matches!(
        parse_lsn("1/100000000"),
        Err(ProtocolError::InvalidLsn { .. })
    ));
    assert!(matches!(
        parse_lsn("100000000/1"),
        Err(ProtocolError::InvalidLsn { .. })
    ));
    assert!(matches!(
        parse_lsn("/1"),
        Err(ProtocolError::InvalidLsn { .. })
    ));
}
