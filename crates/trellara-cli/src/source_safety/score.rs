use crate::{FlowAlertSeverity, FlowHealthStatus, SourceSafetyFactor, SourceSafetyGrade};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceSafetyScore {
    pub(crate) score: u8,
    pub(crate) grade: SourceSafetyGrade,
    pub(crate) status: FlowHealthStatus,
    pub(crate) factor_count: usize,
    pub(crate) critical_factor_count: usize,
    pub(crate) warning_factor_count: usize,
    pub(crate) recommended_actions: Vec<String>,
}

impl SourceSafetyScore {
    pub(crate) fn from_factors(factors: &[SourceSafetyFactor]) -> Self {
        let points_lost = factors
            .iter()
            .map(|factor| u16::from(factor.points_lost))
            .sum::<u16>()
            .min(100);
        let score = (100 - points_lost) as u8;
        let critical_factor_count = factors
            .iter()
            .filter(|factor| factor.severity == FlowAlertSeverity::Critical)
            .count();
        let warning_factor_count = factors.len() - critical_factor_count;
        let status = if critical_factor_count > 0 {
            FlowHealthStatus::Blocked
        } else if warning_factor_count > 0 {
            FlowHealthStatus::Degraded
        } else {
            FlowHealthStatus::Healthy
        };
        let recommended_actions = factors
            .iter()
            .map(|factor| factor.recommendation.clone())
            .collect::<Vec<_>>();

        Self {
            score,
            grade: SourceSafetyGrade::from_score(score),
            status,
            factor_count: factors.len(),
            critical_factor_count,
            warning_factor_count,
            recommended_actions,
        }
    }
}

impl SourceSafetyGrade {
    pub(crate) fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::A,
            75..=89 => Self::B,
            60..=74 => Self::C,
            40..=59 => Self::D,
            _ => Self::F,
        }
    }
}
