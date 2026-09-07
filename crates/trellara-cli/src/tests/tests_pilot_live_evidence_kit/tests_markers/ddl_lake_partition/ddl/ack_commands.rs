use super::*;

#[test]
fn ddl_release_proof_rejects_incomplete_ack_commands() {
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "release_dml true",
        r#"{"release_decision":{"release_dml":false},"ack_commands":[]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{"release_decision":{"release_dml":true},"ack_commands":[]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{"release_decision":{"release_dml":true},"ack_commands":["operator will acknowledge sinks"]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{"release_decision":{"release_dml":true},"ack_commands":["trellara schema ddl-barrier ack --config trellara.yml"]}"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake", "raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres", "raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2",
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-old --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2"
            ]
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "ddl_release_proof",
        "ack_commands",
        r#"{
            "barrier_id": "ddl-barrier-abc",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "release_decision": {"release_dml": true},
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v1"
            ]
        }"#
    ));
}
