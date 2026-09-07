use super::*;

#[test]
fn fleet_evidence_plan_text_renders_command_sequence() {
    let summary = FleetEvidencePlanSummary {
        verdict: "ready_to_collect_live_evidence".to_string(),
        flow_count: 1,
        gate_count: 8,
        needs_live_evidence_gate_count: 4,
        blocked_gate_count: 0,
        required_live_evidence_artifact_count: 4,
        command_count: 2,
        flows: vec![FleetEvidencePlanFlow {
            flow_id: "source-a:sales".to_string(),
            config: "sales.yml".to_string(),
            verdict: "ready_for_live_pilot".to_string(),
            needs_live_evidence_gate_count: 1,
            blocked_gate_count: 0,
            required_live_evidence_artifact_count: 1,
            live_evidence_gates: vec![FleetEvidencePlanGate {
                code: "source_safety".to_string(),
                title: "read-only CDC source-safety assessment".to_string(),
                artifact: "source-safety.txt".to_string(),
                proof_command: "trellara check --config sales.yml --format text".to_string(),
                success_evidence: "no critical blockers".to_string(),
                success_markers: vec!["no critical findings".to_string()],
                collection_requirements: vec![
                    "include findings context proving there are no critical source-safety factors"
                        .to_string(),
                ],
            }],
            blocked_gates: Vec::new(),
        }],
        command_sequence: vec![
            "trellara check --config sales.yml --format text".to_string(),
            "trellara pilot-package --config sales.yml".to_string(),
        ],
        review_rule: "collect every needs_live_evidence command".to_string(),
        next_commands: vec!["trellara fleet scorecard --config sales.yml".to_string()],
    };

    let output = render_fleet_evidence_plan_text(&summary);

    assert!(output.contains("Trellara fleet evidence plan"));
    assert!(output.contains("verdict: ready_to_collect_live_evidence"));
    assert!(output.contains("required_live_artifacts: 4"));
    assert!(output.contains("required_live_artifacts=1"));
    assert!(output.contains("[needs_live_evidence] source_safety"));
    assert!(output.contains("artifact=source-safety.txt"));
    assert!(output.contains("success_markers: no critical findings"));
    assert!(output.contains("requirements: include findings context"));
    assert!(output.contains("command_sequence:"));
    assert!(output.contains("trellara pilot-package --config sales.yml"));
}
