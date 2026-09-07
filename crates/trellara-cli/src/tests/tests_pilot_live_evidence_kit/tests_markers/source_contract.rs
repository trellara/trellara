use super::*;

#[test]
fn contract_preflight_requires_structured_passing_checks() {
    let proof = r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "passed": true,
        "check_count": 2,
        "issue_count": 0,
        "recovery_action_count": 0,
        "checks": [
            {
                "name": "source_and_target_contract:public.sales",
                "passed": true,
                "severity": "info",
                "message": "public.sales satisfies source capture and target compatibility checks",
                "recommendation": ""
            },
            {
                "name": "transaction_boundary:strict",
                "passed": true,
                "severity": "warning",
                "message": "strict transaction boundary includes advisory latency cost",
                "recommendation": ""
            }
        ],
        "recovery_actions": []
    }"#;

    for marker in live_evidence_expected_markers("contract_preflight") {
        assert!(
            live_evidence_marker_present("contract_preflight", &marker, proof),
            "expected marker {marker}"
        );
    }

    assert!(!live_evidence_marker_present(
        "contract_preflight",
        "source dataset identity",
        r#"{"dataset_id":"retail-sales","passed":true,"check_count":1,"issue_count":0,"checks":[{"name":"source","passed":true,"severity":"info","message":"ok"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "contract_preflight",
        "source dataset identity",
        r#"{"source_id":"local-source","dataset_id":"unknown","passed":true,"check_count":1,"issue_count":0,"checks":[{"name":"source","passed":true,"severity":"info","message":"ok"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "contract_preflight",
        "passed true",
        r#"{"passed":true}"#
    ));
    assert!(!live_evidence_marker_present(
        "contract_preflight",
        "checks present",
        "ready=true"
    ));
    assert!(!live_evidence_marker_present(
        "contract_preflight",
        "check counts consistent",
        r#"{"passed":true,"check_count":2,"issue_count":0,"checks":[{"name":"source","passed":true,"severity":"info","message":"ok"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "contract_preflight",
        "no failed or error checks",
        r#"{"passed":true,"check_count":1,"issue_count":0,"checks":[{"name":"source","passed":true,"severity":"error","message":"contradictory"}]}"#
    ));
}

#[test]
fn source_safety_live_evidence_accepts_actual_noncritical_statuses() {
    let healthy =
        "Trellara source safety\nsource: local-source\ndataset: retail-sales\nstatus: healthy\ngrade: A (100/100)\n\nfindings:\n- none\n";
    for marker in live_evidence_expected_markers("source_safety") {
        assert!(
            live_evidence_marker_present("source_safety", &marker, healthy),
            "expected text marker {marker}"
        );
    }
    assert!(live_evidence_marker_present(
            "source_safety",
            "ready_or_warning_status",
            "Trellara source safety\nsource: local-source\ndataset: retail-sales\nstatus: degraded\ngrade: B (85/100)\n\nfindings:\n- [warning] source_slot_failover_disabled\n"
        ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "source dataset identity",
        "Trellara source safety\nstatus: degraded\ngrade: B (85/100)\n\nfindings:\n- [warning] source_slot_failover_disabled\n"
    ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "source dataset identity",
        r#"{"source_id":"local-source","dataset_id":"unknown","status":"healthy","factor_count":0,"critical_factor_count":0,"factors":[]}"#
    ));
    let json = r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "status": "degraded",
        "factor_count": 1,
        "critical_factor_count": 0,
        "factors": [
            {
                "code": "source_slot_failover_disabled",
                "severity": "warning",
                "evidence": "slot failover=false on a source with failover readiness enabled",
                "recommendation": "enable failover slots before promotion drills"
            }
        ]
    }"#;
    for marker in live_evidence_expected_markers("source_safety") {
        assert!(
            live_evidence_marker_present("source_safety", &marker, json),
            "expected json marker {marker}"
        );
    }
    assert!(!live_evidence_marker_present(
        "source_safety",
        "ready_or_warning_status",
        "Trellara source safety\nstatus: blocked\ngrade: F (30/100)\n"
    ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "no critical findings",
        "Trellara source safety\nstatus: healthy\ngrade: A (100/100)\n"
    ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "no critical findings",
        "Trellara source safety\nstatus: healthy\ngrade: A (100/100)\n\nfindings:\n"
    ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "no critical findings",
        "Trellara source safety\nstatus: healthy\ngrade: A (100/100)\n\nfindings:\noperator reviewed source posture\n"
    ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "no critical findings",
        r#"{"source_id":"local-source","dataset_id":"retail-sales","status":"healthy","critical_factor_count":0}"#
    ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "no critical findings",
        r#"{"source_id":"local-source","dataset_id":"retail-sales","status":"healthy","factor_count":1,"critical_factor_count":0,"factors":[{"severity":"critical"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "no critical findings",
        r#"{"source_id":"local-source","dataset_id":"retail-sales","status":"degraded","factor_count":2,"critical_factor_count":0,"factors":[{"code":"source_slot_failover_disabled","severity":"warning","evidence":"slot failover=false","recommendation":"enable failover slot"}]}"#
    ));
    assert!(!live_evidence_marker_present(
        "source_safety",
        "no critical findings",
        r#"{"source_id":"local-source","dataset_id":"retail-sales","status":"degraded","factor_count":1,"critical_factor_count":0,"factors":[{"code":"source_slot_failover_disabled","severity":"warning","evidence":"","recommendation":"enable failover slot"}]}"#
    ));
}
