use super::*;

#[test]
fn ddl_plan_blocks_new_table_when_unknown_table_policy_rejects() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let summary = DdlPlanSummary::from_args(
        &config,
        &DdlPlanArgs {
            config: PathBuf::from("strict.yml"),
            changes: vec!["add_table:public.refunds".to_string()],
            apply_mode: DdlPlanApplyMode::AutoSafe,
            format: QuickstartOutputFormat::Json,
        },
    )
    .expect("ddl plan");

    assert_eq!(summary.verdict, DdlPlanVerdict::Blocked);
    assert_eq!(summary.blocked_count, 1);
    assert_eq!(summary.blockers.len(), 1);
    assert_eq!(summary.blockers[0].change, "add_table:public.refunds");
    assert_eq!(
        summary.blockers[0].compatibility,
        DdlPlanCompatibility::BlockedByPolicy
    );
    assert!(summary.blockers[0]
        .release_impact
        .contains("blocks post-DDL DML release"));
    assert!(summary
        .propagation
        .release_blockers
        .iter()
        .any(|blocker| blocker.contains("schema policy blockers")));
    assert!(summary.propagation.policy_modes.iter().any(|policy| {
        policy.mode == DdlPropagationPolicyMode::BlockUnsupported
            && policy.active
            && policy.change_count == 1
            && policy
                .approval_evidence
                .contains("blocker remediation evidence")
    }));
    assert!(summary.propagation.policy_modes.iter().any(|policy| {
        policy.mode == DdlPropagationPolicyMode::ShadowPlanOnly
            && policy.active
            && policy
                .release_rule
                .contains("review and repair evidence only")
    }));
    assert!(summary.propagation.release_blockers.iter().any(|blocker| {
        blocker.contains("add_table on public.refunds")
            && blocker.contains("unknown_table_policy=reject")
    }));
    assert_eq!(
        summary.changes[0].compatibility,
        DdlPlanCompatibility::BlockedByPolicy
    );
    assert!(summary.changes[0]
        .reason
        .contains("unknown_table_policy=reject"));
    assert!(summary
        .next_commands
        .iter()
        .any(|command| command.contains("repair-plan")));
    assert!(!summary
        .next_commands
        .iter()
        .any(|command| command.contains("ddl-barrier record")));
    let text = render_ddl_plan_text(&summary);
    assert!(text.contains("blockers:"));
    assert!(text.contains("add_table:public.refunds"));
    assert!(text.contains("unknown_table_policy=reject"));
    assert!(text.contains("approval_evidence="));
    assert!(text.contains("shadow plan review artifact"));
}
