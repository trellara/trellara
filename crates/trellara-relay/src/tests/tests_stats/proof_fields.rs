use super::*;

#[test]
fn relay_run_stats_rejects_source_ack_lsn_proof_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.source_ack_lsn = "0/16B6C51".to_string();

    let error = stats
        .record_step(step)
        .expect_err("source ack proof mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "source_ack_lsn",
            step,
            proof,
        } if step == "\"0/16B6C50\"" && proof == "\"0/16B6C51\""
    ));
}

#[test]
fn relay_run_stats_rejects_source_ack_contract_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.contract = "source ack after checkpoint only";

    let error = stats
        .record_step(step)
        .expect_err("source ack contract mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "contract",
            step,
            proof,
        } if step.contains("source acknowledgement advances only after every Trellara publish ack")
            && proof == "\"source ack after checkpoint only\""
    ));
}

#[test]
fn relay_run_stats_rejects_unsatisfied_source_ack_proof_gate() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.all_publish_acks_durable = false;

    let error = stats
        .record_step(step)
        .expect_err("unsatisfied source ack proof gate");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "all_publish_acks_durable",
            step,
            proof,
        } if step == "true" && proof == "false"
    ));
}

#[test]
fn relay_run_stats_rejects_expected_publish_message_proof_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.expected_publish_messages = 2;

    let error = stats
        .record_step(step)
        .expect_err("expected publish message proof mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "expected_publish_messages",
            step,
            proof,
        } if step == "1" && proof == "2"
    ));
}

#[test]
fn relay_run_stats_rejects_last_publish_ack_proof_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.last_publish_ack = Some(ack(8));

    let error = stats
        .record_step(step)
        .expect_err("last publish ack proof mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "last_publish_ack",
            step,
            proof,
        } if step.contains("offset: 7") && proof.contains("offset: 8")
    ));
}
