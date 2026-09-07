use super::*;

#[test]
fn protocol_rfc_tracks_public_version_and_boundary_modes() {
    let rfc = include_str!("../../../../docs/DESIGN.md");

    assert!(rfc.contains(&format!("Protocol version: `{PROTOCOL_VERSION}`")));
    assert!(rfc.contains("Strict Transaction Order"));
    assert!(rfc.contains("Strict Chunked Transaction Order"));
    assert!(rfc.contains("Partitioned Scale Mode"));
    assert!(rfc.contains("{source_id}:{commit_lsn}:{transaction_id}:{total_order}"));
    assert!(rfc.contains("Empty transactions are not valid manifest-barrier payloads"));
    assert!(rfc.contains(
        "publish durable message(s) -> save source durable checkpoint -> acknowledge source LSN"
    ));
    assert!(
        rfc.contains("apply target changes + record transaction dedup + save target checkpoint")
    );
}
