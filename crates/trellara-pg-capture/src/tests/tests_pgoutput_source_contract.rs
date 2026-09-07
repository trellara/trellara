use super::*;
use crate::source_ack_boundary::DurableSourceAckBoundary;

#[test]
fn pgoutput_stream_capture_exposes_production_source_contract() {
    fn assert_contract<T: PgChangeSource + ChangeSource + Send>() {}

    assert_contract::<PgOutputStreamCapture>();
}

#[test]
fn capture_bootstrap_preserves_stream_start_evidence() {
    let bootstrap = CaptureBootstrap {
        publication_name: "trellara_pub".to_string(),
        slot_name: "trellara_slot".to_string(),
        consistent_lsn: Some("0/16B6C50".to_string()),
        exported_snapshot_name: None,
        relations: vec![CapturedRelation {
            id: RelationId::new(42, "public", "sales"),
            replica_identity: trellara_protocol::ReplicaIdentity::Default,
        }],
        preflight: Vec::new(),
    };

    assert_eq!(bootstrap.publication_name, "trellara_pub");
    assert_eq!(bootstrap.slot_name, "trellara_slot");
    assert_eq!(bootstrap.consistent_lsn.as_deref(), Some("0/16B6C50"));
    assert_eq!(bootstrap.relations[0].id.display_name(), "public.sales");
}

#[test]
fn durable_source_ack_boundary_canonicalizes_forward_progress() {
    let mut boundary = DurableSourceAckBoundary::default();

    assert_eq!(boundary.acknowledge("0/16b6c50").expect("ack"), 0x16B6C50);
    assert_eq!(boundary.last_acknowledged_lsn(), 0x16B6C50);
    assert_eq!(
        boundary.acknowledge("0/16B6C60").expect("next ack"),
        0x16B6C60
    );
}

#[test]
fn durable_source_ack_boundary_allows_duplicate_ack() {
    let mut boundary = DurableSourceAckBoundary::default();

    boundary.acknowledge("0/16B6C50").expect("first ack");

    assert_eq!(
        boundary.acknowledge("0/16B6C50").expect("duplicate ack"),
        0x16B6C50
    );
}

#[test]
fn durable_source_ack_boundary_rejects_zero_lsn() {
    let mut boundary = DurableSourceAckBoundary::default();

    assert!(matches!(
        boundary.acknowledge("0/0"),
        Err(CaptureError::ReplicationProtocol(message))
            if message.contains("cannot acknowledge durable source LSN 0/0")
    ));
}

#[test]
fn durable_source_ack_boundary_rejects_backward_progress() {
    let mut boundary = DurableSourceAckBoundary::default();

    boundary.acknowledge("0/16B6C60").expect("first ack");

    assert!(matches!(
        boundary.acknowledge("0/16B6C50"),
        Err(CaptureError::ReplicationProtocol(message))
            if message.contains("cannot move durable source acknowledgement backward")
                && message.contains("0/16B6C60")
                && message.contains("0/16B6C50")
    ));
    assert_eq!(boundary.last_acknowledged_lsn(), 0x16B6C60);
}
