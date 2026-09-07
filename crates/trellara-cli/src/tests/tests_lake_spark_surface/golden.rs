use super::*;

#[test]
fn spark_golden_fixture_derives_current_state_and_scd2_outputs() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let fixture = SparkGoldenFixture::from_config(&config);

    assert_eq!(fixture.epoch.state, "complete");
    assert_eq!(fixture.verification.checksum_status, "match");
    assert_eq!(fixture.raw_cdc.len(), 4);
    assert_eq!(
        fixture.expected_current_state,
        materialize_current_state(&fixture.raw_cdc)
    );
    assert_eq!(fixture.expected_current_state.len(), 1);
    assert_eq!(fixture.expected_current_state[0].record_key, "sale-100");
    assert!(fixture.expected_current_state[0]
        .row_after_json
        .contains(r#""status":"paid""#));

    assert_eq!(fixture.expected_scd2, materialize_scd2(&fixture.raw_cdc));
    assert_eq!(fixture.expected_scd2.len(), 3);
    assert!(fixture
        .expected_scd2
        .iter()
        .any(|row| row.record_key == "sale-100" && !row.is_current));
    assert!(fixture
        .expected_scd2
        .iter()
        .any(|row| row.record_key == "sale-100" && row.is_current));
    assert!(fixture
        .expected_scd2
        .iter()
        .any(|row| row.record_key == "sale-200" && !row.is_current));
}

#[test]
fn spark_golden_fixture_documents_idempotent_epoch_reruns() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let fixture = SparkGoldenFixture::from_config(&config);
    let json = serde_json::to_string_pretty(&fixture).expect("serialize golden fixture");

    assert!(json.contains("__trellara_idempotency_key"));
    assert!(json.contains("__trellara_payload_before"));
    assert!(json.contains("__trellara_payload_after"));
    assert!(json.contains("expected_current_state"));
    assert!(json.contains("expected_scd2"));
    assert!(json.contains("rerunning the same epoch must produce the same"));
    assert!(!json.contains("${"));
}
