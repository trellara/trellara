#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FailureInjection {
    injected: bool,
    description: Option<String>,
}

impl FailureInjection {
    pub(crate) fn should_inject<F>(&mut self, configured: F, target: F) -> bool
    where
        F: Eq,
    {
        if self.injected || configured != target {
            return false;
        }
        self.injected = true;
        true
    }

    pub(crate) fn mark(&mut self, failure: impl Into<String>) {
        self.description = Some(failure.into());
    }

    pub(crate) fn into_description(self) -> Option<String> {
        self.description
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum FailurePoint {
        BeforeCommit,
        AfterCommit,
    }

    #[test]
    fn injects_only_once_for_matching_failure_point() {
        let mut injection = FailureInjection::default();

        assert!(injection.should_inject(FailurePoint::BeforeCommit, FailurePoint::BeforeCommit));
        assert!(!injection.should_inject(FailurePoint::BeforeCommit, FailurePoint::BeforeCommit));
        assert!(!injection.should_inject(FailurePoint::BeforeCommit, FailurePoint::AfterCommit));
    }

    #[test]
    fn keeps_operator_visible_failure_description() {
        let mut injection = FailureInjection::default();

        injection.mark("target failed before commit");

        assert_eq!(
            injection.into_description(),
            Some("target failed before commit".to_string())
        );
    }
}
