use super::*;

#[test]
fn idempotency_key_is_deterministic() {
    assert_eq!(
        idempotency_key("source-a", "0/16B6C50", "tx-1", 42),
        idempotency_key("source-a", "0/16B6C50", "tx-1", 42)
    );
}
