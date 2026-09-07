use super::*;

#[test]
fn quarantine_transaction_key_rejects_empty_operator_inputs() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse config");

    let error = quarantine_transaction_key(
        &config,
        "quarantine.clear",
        " ".to_string(),
        "0/16B6D28".to_string(),
    )
    .expect_err("empty transaction id rejected");

    assert!(error
        .to_string()
        .contains("quarantine.clear.transaction_id must not be empty"));

    let error = quarantine_transaction_key(
        &config,
        "quarantine.replay_ready",
        "tx-blocked".to_string(),
        " ".to_string(),
    )
    .expect_err("empty commit lsn rejected");

    assert!(error
        .to_string()
        .contains("quarantine.replay_ready.commit_lsn must not be empty"));
}

#[test]
fn quarantine_transaction_key_rejects_operator_inputs_with_surrounding_whitespace() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse config");

    let error = quarantine_transaction_key(
        &config,
        "quarantine.clear",
        " tx-blocked ".to_string(),
        "0/16B6D28".to_string(),
    )
    .expect_err("spaced transaction id rejected");

    assert!(error
        .to_string()
        .contains("quarantine.clear.transaction_id must not contain surrounding whitespace"));

    let error = quarantine_transaction_key(
        &config,
        "quarantine.replay_ready",
        "tx-blocked".to_string(),
        " 0/16B6D28 ".to_string(),
    )
    .expect_err("spaced commit lsn rejected");

    assert!(error
        .to_string()
        .contains("quarantine.replay_ready.commit_lsn must not contain surrounding whitespace"));
}

#[test]
fn quarantine_transaction_key_rejects_invalid_commit_lsn_before_store_access() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse config");

    let error = quarantine_transaction_key(
        &config,
        "quarantine.replay_ready",
        "tx-blocked".to_string(),
        "0/0".to_string(),
    )
    .expect_err("zero commit lsn rejected");

    assert!(error.to_string().contains("checkpoint error"));
    assert!(error.to_string().contains("commit_lsn"));
    assert!(error.to_string().contains("LSN must be greater than zero"));
}
