use super::*;

pub(super) fn local_mvp_summary() -> MvpReadinessSummary {
    let config_path = workspace_path("examples/retail-fleet/local.yml");
    let config = TrellaraConfig::from_path(&config_path).expect("parse config");

    MvpReadinessSummary::from_config(&config, &config_path).expect("mvp readiness")
}

pub(super) fn criterion<'a>(
    summary: &'a MvpReadinessSummary,
    code: &str,
) -> &'a MvpReadinessCriterion {
    summary
        .criteria
        .iter()
        .find(|criterion| criterion.code == code)
        .unwrap_or_else(|| panic!("missing {code} criterion"))
}

pub(super) fn assert_contains_all(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "expected `{haystack}` to contain `{needle}`"
        );
    }
}

pub(super) fn assert_passed_criterion<'a>(
    summary: &'a MvpReadinessSummary,
    code: &str,
) -> &'a MvpReadinessCriterion {
    let criterion = criterion(summary, code);
    assert!(criterion.passed, "{code} should pass");
    criterion
}
