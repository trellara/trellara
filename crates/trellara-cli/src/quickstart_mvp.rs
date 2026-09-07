use std::path::Path;

use crate::quickstart_mvp_criteria::build_mvp_readiness_criteria;
use crate::{
    mvp_readiness_next_commands, mvp_readiness_priority_next_commands, quickstart_readiness,
    repository_root_for, ChaosRunSummary, MvpProofCounts, MvpReadinessCriterion,
    MvpReadinessSummary, QuickstartArgs, QuickstartOutputFormat, Result, TrellaraConfig,
};

impl MvpReadinessSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, config_path: &Path) -> Result<Self> {
        let config_display = config_path.display().to_string();
        let repository_root = repository_root_for(config_path);
        let quickstart = quickstart_readiness(&QuickstartArgs {
            config: config_path.to_path_buf(),
            check: true,
            format: QuickstartOutputFormat::Json,
        })?;
        let chaos = ChaosRunSummary::default();
        let proof_counts = MvpProofCounts::from_chaos(&chaos);

        let criteria = build_mvp_readiness_criteria(
            config,
            &config_display,
            &repository_root,
            &quickstart,
            &chaos,
            &proof_counts,
        );

        let passed_criterion_count = criteria.iter().filter(|criterion| criterion.passed).count();
        let ready = passed_criterion_count == criteria.len();
        let priority_next_commands =
            mvp_readiness_priority_next_commands(&config_display, &criteria, ready);
        let next_commands = mvp_readiness_next_commands(&config_display, &criteria, ready);

        Ok(Self {
            config: config_display,
            ready,
            criterion_count: criteria.len(),
            passed_criterion_count,
            criteria,
            priority_next_commands,
            next_commands,
        })
    }
}

impl MvpReadinessCriterion {
    pub(crate) fn new(
        code: impl Into<String>,
        passed: bool,
        evidence: impl Into<String>,
        proof_command: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            passed,
            evidence: evidence.into(),
            proof_command: proof_command.into(),
        }
    }
}
