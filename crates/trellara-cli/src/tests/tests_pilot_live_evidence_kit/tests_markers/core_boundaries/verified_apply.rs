use super::*;

#[test]
fn verified_apply_requires_watermarks_and_table_evidence() {
    let proof = "source_id=local-source dataset_id=retail-sales converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1 relation=public.sales target_relation=public.sales relation_match=true";
    for marker in live_evidence_expected_markers("verified_apply") {
        assert!(
            live_evidence_marker_present("verified_apply", &marker, proof),
            "expected marker {marker}"
        );
    }

    let json = r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "source_watermark_lsn": "0/16B6C50",
        "target_watermark_lsn": "0/16B6C50",
        "converged": true,
        "checksum_status": "match",
        "table_count": 1,
        "tables": [
            {
                "relation": "public.sales",
                "target_relation": "public.sales",
                "relation_match": true,
                "converged": true,
                "checksum_status": "match"
            }
        ]
    }"#;
    for marker in live_evidence_expected_markers("verified_apply") {
        assert!(
            live_evidence_marker_present("verified_apply", &marker, json),
            "expected json marker {marker}"
        );
    }

    assert!(!live_evidence_marker_present(
        "verified_apply",
        "source dataset identity",
        "dataset_id=retail-sales converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1 relation=public.sales"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "source dataset identity",
        r#"{"source_id":"local-source","dataset_id":"","source_watermark_lsn":"0/16B6C50","target_watermark_lsn":"0/16B6C50","converged":true,"checksum_status":"match","table_count":1,"tables":[{"relation":"public.sales","converged":true,"checksum_status":"match"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "converged true",
        "converged=true\nchecksum_status=match\n"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=0"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        r#"{"source_watermark_lsn":"0/16B6C50","target_watermark_lsn":"0/16B6C50","converged":true,"checksum_status":"match","table_count":0,"tables":[]}"#
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        r#"{"source_watermark_lsn":"0/16B6C50","target_watermark_lsn":"0/16B6C50","converged":true,"checksum_status":"match","tables":[{"relation":"public.sales","converged":true,"checksum_status":"match"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        r#"{"source_watermark_lsn":"0/16B6C50","target_watermark_lsn":"0/16B6C50","converged":true,"checksum_status":"match","table_count":2,"tables":[{"relation":"public.sales","converged":true,"checksum_status":"match"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "converged true",
        "converged=true checksum_status=match source_watermark_lsn=not-a-lsn target_watermark_lsn=0/16B6C50 table_count=1"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B8000 table_count=1"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        r#"{"source_watermark_lsn":"0/16B6C50","target_watermark_lsn":"0/16B6C50","converged":true,"checksum_status":"match","table_count":1,"tables":[{"relation":"public.sales","converged":false,"checksum_status":"match"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        r#"{"source_watermark_lsn":"0/16B6C50","target_watermark_lsn":"0/16B6C50","converged":true,"checksum_status":"match","table_count":1,"tables":[{"relation":"public.sales","target_relation":"archive.sales","relation_match":false,"converged":true,"checksum_status":"match"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        r#"{"source_watermark_lsn":"0/16B6C50","target_watermark_lsn":"0/16B6C50","converged":true,"checksum_status":"match","table_count":1,"tables":[{"relation":"public.sales","target_relation":"archive.sales","converged":true,"checksum_status":"match"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1 relation=public.sales target_relation=archive.sales relation_match=false"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1 relation=public.sales target_relation=archive.sales"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "target relation identity",
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1 relation=public.sales"
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "target relation identity",
        r#"{"source_watermark_lsn":"0/16B6C50","target_watermark_lsn":"0/16B6C50","converged":true,"checksum_status":"match","table_count":1,"tables":[{"relation":"public.sales","converged":true,"checksum_status":"match"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "verified_apply",
        "checksum match",
        "converged=false checksum_status=mismatch source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B8000 table_count=1"
    ));
}
