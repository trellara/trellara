use super::*;

#[test]
fn relay_run_stats_rejects_durable_publish_destination_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.durable_publish_destinations[0].topic = "wrong-topic".to_string();

    let error = stats
        .record_step(step)
        .expect_err("durable publish destination proof mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "durable_publish_destinations",
            step,
            proof,
        } if step.contains("topic") && proof.contains("wrong-topic")
    ));
}

#[test]
fn relay_run_stats_rejects_expected_publish_destination_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.expected_publish_destinations[0].topic = "wrong-topic".to_string();

    let error = stats
        .record_step(step)
        .expect_err("expected publish destination proof mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "expected_publish_destinations",
            step,
            proof,
        } if step.contains("topic") && proof.contains("wrong-topic")
    ));
}

#[test]
fn relay_run_stats_rejects_destination_count_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.expected_publish_destination_count = 2;

    let error = stats
        .record_step(step)
        .expect_err("expected destination count proof mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "expected_publish_destination_count",
            step,
            proof,
        } if step == "1" && proof == "2"
    ));
}

#[test]
fn relay_run_stats_rejects_durable_destination_count_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.durable_publish_destination_count = 2;

    let error = stats
        .record_step(step)
        .expect_err("durable destination count proof mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "durable_publish_destination_count",
            step,
            proof,
        } if step == "1" && proof == "2"
    ));
}

#[test]
fn relay_run_stats_rejects_destination_match_flag_mismatch() {
    let mut stats = RelayRunStats::default();
    let mut step = relay_step(vec![ack(7)]);
    step.source_ack_boundary.publish_destinations_match = false;

    let error = stats
        .record_step(step)
        .expect_err("destination match flag proof mismatch");

    assert!(matches!(
        error,
        RelayError::SourceAckProofMismatch {
            field: "publish_destinations_match",
            step,
            proof,
        } if step == "true" && proof == "false"
    ));
}
