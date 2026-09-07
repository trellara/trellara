use super::*;

#[test]
fn snapshot_handoff_requires_complete_tables_at_consistent_lsn() {
    let proof = r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "run_id": "pilot-snapshot-1",
        "state": "stream_handoff_ready",
        "slot": "trellara_retail_sales",
        "consistent_lsn": "0/16B6C50",
        "selected_table_count": 1,
        "table_count": 1,
        "skipped_table_count": 0,
        "copied_rows": 3,
        "tables": [
            {
                "relation": "public.sales",
                "state": "copy_complete",
                "copied_rows": 3,
                "skipped": false,
                "watermark_lsn": "0/16B6C50"
            }
        ],
        "next_commands": ["trellara relay --config <config>", "trellara apply --config <config>", "trellara verify --config <config>"],
        "consistency_note": "copies source tables inside the exported logical snapshot held by the pgoutput replication connection, then hands off at the slot consistent LSN"
    }"#;

    for marker in live_evidence_expected_markers("snapshot_handoff") {
        assert!(
            live_evidence_marker_present("snapshot_handoff", &marker, proof),
            "expected marker {marker}"
        );
    }

    let text_proof = "source_id=local-source dataset_id=retail-sales state=stream_handoff_ready selected_table_count=1 table_count=1 public.sales copy_complete consistent_lsn=0/16B6C50 watermark_lsn=0/16B6C50";
    for marker in live_evidence_expected_markers("snapshot_handoff") {
        assert!(
            live_evidence_marker_present("snapshot_handoff", &marker, text_proof),
            "expected text marker {marker}"
        );
    }

    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "source dataset identity",
        "dataset_id=retail-sales state=stream_handoff_ready selected_table_count=1 table_count=1 public.sales copy_complete consistent_lsn=0/16B6C50 watermark_lsn=0/16B6C50"
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "source dataset identity",
        r#"{"source_id":"local-source","dataset_id":"unknown","state":"stream_handoff_ready","run_id":"pilot-snapshot-1","slot":"trellara_retail_sales","consistent_lsn":"0/16B6C50","selected_table_count":1,"table_count":1,"tables":[{"relation":"public.sales","state":"copy_complete","watermark_lsn":"0/16B6C50"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "stream_handoff_ready",
        "public.sales copy_complete\nstate=stream_handoff_ready\nhandoff watermark: 0/16B6C50\n"
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "copy_complete",
        r#"{"state":"stream_handoff_ready","consistent_lsn":"0/16B6C50","selected_table_count":1,"table_count":1,"tables":[]}"#
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "durable handoff watermark",
        r#"{"state":"stream_handoff_ready","consistent_lsn":"not-a-lsn","selected_table_count":1,"table_count":1,"tables":[{"relation":"public.sales","state":"copy_complete","watermark_lsn":"not-a-lsn"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "stream_handoff_ready",
        "state=stream_handoff_ready selected_table_count=1 table_count=1 public.sales copy_complete consistent_lsn=0/16B6C50 handoff watermark: 0/not-lsn"
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "durable handoff watermark",
        "state=stream_handoff_ready selected_table_count=1 table_count=1 public.sales copy_complete consistent_lsn=0/16B6C50 watermark_lsn=0/16B9000"
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "durable handoff watermark",
        r#"{"state":"stream_handoff_ready","consistent_lsn":"0/16B6C50","selected_table_count":1,"table_count":1,"tables":[{"relation":"public.sales","state":"copy_complete","watermark_lsn":"0/16B9000"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "selected table coverage",
        r#"{"state":"stream_handoff_ready","consistent_lsn":"0/16B6C50","selected_table_count":2,"table_count":1,"tables":[{"relation":"public.sales","state":"copy_complete","watermark_lsn":"0/16B6C50"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "snapshot_handoff",
        "selected table coverage",
        "state=stream_handoff_ready selected_table_count=2 table_count=1 public.sales copy_complete consistent_lsn=0/16B6C50 watermark_lsn=0/16B6C50"
    ));
}
