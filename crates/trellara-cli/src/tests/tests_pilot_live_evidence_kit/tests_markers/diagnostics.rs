use super::*;

#[test]
fn failure_drill_requires_diagnostics_repair_and_quarantine_surfaces() {
    let diagnostics = "Trellara diagnostics\nconfig: trellara.yml\nsource: local-source\ndataset: retail-sales\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\nalerts: 0\nrepair_plan_required: false\n\nattachment_commands:\n- trellara status --config trellara.yml --view diagnostics --format text\n- trellara repair-plan --config trellara.yml\n- trellara quarantine list --config trellara.yml\n\nrecommended_actions:\n- run trellara verify after repair; use trellara reseed if checksum drift remains\n";
    for marker in live_evidence_expected_markers("failure_drill") {
        assert!(
            live_evidence_marker_present("failure_drill", &marker, diagnostics),
            "expected text marker {marker}"
        );
    }

    let json = r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "mode": "strict_chunked_transaction_order",
        "config": "trellara.yml",
        "ready": false,
        "status": "healthy",
        "report": {
            "no_target_quarantine": true,
            "recommended_actions": ["run trellara verify after repair"],
            "proof_checks": [
                {"code": "target_quarantine", "status": "verified", "evidence": "target quarantine has no blocked transaction"}
            ]
        },
        "repair_plan": {
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "dry_run": true,
            "plan_required": false,
            "step_count": 0,
            "steps": []
        },
        "attachment_commands": [
            "trellara status --config trellara.yml --view diagnostics --format text",
            "trellara repair-plan --config trellara.yml",
            "trellara quarantine list --config trellara.yml"
        ]
    }"#;
    for marker in live_evidence_expected_markers("failure_drill") {
        assert!(
            live_evidence_marker_present("failure_drill", &marker, json),
            "expected json marker {marker}"
        );
    }

    assert!(!live_evidence_marker_present(
        "failure_drill",
        "source dataset identity",
        "Trellara diagnostics\nconfig: trellara.yml\ndataset: retail-sales\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\n"
    ));
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "source dataset identity",
        r#"{"source_id":"local-source","dataset_id":"unknown","mode":"strict_chunked_transaction_order","config":"trellara.yml","ready":false,"status":"healthy","report":{}}"#
    ));
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "Trellara diagnostics",
        "Trellara diagnostics\nrepair_plan_required: true\nquarantine list captured\n"
    ));
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "repair_plan_required",
        "Trellara diagnostics\nconfig: trellara.yml\nsource: local-source\ndataset: retail-sales\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\nrepair_plan_required: true\n"
    ));
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "repair_plan_required",
        "Trellara diagnostics\nconfig: trellara.yml\nsource: local-source\ndataset: retail-sales\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\nrepair_plan_required: true\nrecommended_actions:\n- repair target\nattachment_commands:\n- trellara repair-plan --config trellara.yml\n"
    ));
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "repair_plan_required",
        "Trellara diagnostics\nconfig: trellara.yml\nsource: local-source\ndataset: retail-sales\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\nrepair_plan_required: true\nrecommended_actions:\n- run trellara verify after repair\nattachment_commands:\n- trellara status --config trellara.yml\n"
    ));
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "quarantine",
        "Trellara diagnostics\nconfig: trellara.yml\nsource: local-source\ndataset: retail-sales\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\nrepair_plan_required: false\nrecommended_actions:\n- repair target\n"
    ));
}

#[test]
fn failure_drill_json_repair_plan_requires_consistent_actionable_steps() {
    let json = r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "mode": "strict_chunked_transaction_order",
        "config": "trellara.yml",
        "ready": false,
        "status": "warning",
        "report": {
            "no_target_quarantine": true,
            "recommended_actions": ["replay quarantined transaction after repair"],
            "proof_checks": [
                {"code": "target_quarantine", "status": "verified", "evidence": "target quarantine checked"}
            ]
        },
        "repair_plan": {
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "dry_run": true,
            "plan_required": true,
            "step_count": 1,
            "steps": [
                {
                    "order": 1,
                    "action_code": "target_quarantine_replay",
                    "reason": "target transaction is quarantined",
                    "command": "trellara quarantine replay-ready --config trellara.yml --transaction-id tx-1 --commit-lsn 0/16B6C50",
                    "redelivery_topics": [],
                    "redelivery_warnings": [],
                    "hint": "redeliver only after the transaction boundary is replay-ready"
                }
            ]
        },
        "attachment_commands": [
            "trellara quarantine list --config trellara.yml"
        ]
    }"#;
    assert!(live_evidence_marker_present(
        "failure_drill",
        "repair_plan_required",
        json
    ));

    let mismatched_count = json.replace("\"step_count\": 1", "\"step_count\": 2");
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "repair_plan_required",
        &mismatched_count
    ));

    let empty_required_plan = json
        .replace("\"step_count\": 1", "\"step_count\": 0")
        .replace(
            r#""steps": [
                {
                    "order": 1,
                    "action_code": "target_quarantine_replay",
                    "reason": "target transaction is quarantined",
                    "command": "trellara quarantine replay-ready --config trellara.yml --transaction-id tx-1 --commit-lsn 0/16B6C50",
                    "redelivery_topics": [],
                    "redelivery_warnings": [],
                    "hint": "redeliver only after the transaction boundary is replay-ready"
                }
            ]"#,
            r#""steps": []"#,
        );
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "repair_plan_required",
        &empty_required_plan
    ));

    let missing_command = json.replace(
        r#""command": "trellara quarantine replay-ready --config trellara.yml --transaction-id tx-1 --commit-lsn 0/16B6C50""#,
        r#""command": """#,
    );
    assert!(!live_evidence_marker_present(
        "failure_drill",
        "repair_plan_required",
        &missing_command
    ));
}
