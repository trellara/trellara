use crate::{CheckFactor, CheckGrade, CheckSeverity, CheckStatus};

pub(crate) struct CheckScore {
    pub(crate) score: u8,
    pub(crate) grade: CheckGrade,
    pub(crate) status: CheckStatus,
}

pub(crate) fn score_from_factors(findings: &[CheckFactor]) -> CheckScore {
    let points_lost = findings
        .iter()
        .map(|factor| u16::from(factor.points_lost))
        .sum::<u16>()
        .min(100);
    let score = (100 - points_lost) as u8;
    let status = if findings
        .iter()
        .any(|factor| factor.severity == CheckSeverity::Critical)
    {
        CheckStatus::Blocked
    } else if findings.is_empty() {
        CheckStatus::Healthy
    } else {
        CheckStatus::Degraded
    };

    CheckScore {
        score,
        grade: grade_from_score(score),
        status,
    }
}

fn grade_from_score(score: u8) -> CheckGrade {
    match score {
        90..=100 => CheckGrade::A,
        75..=89 => CheckGrade::B,
        60..=74 => CheckGrade::C,
        40..=59 => CheckGrade::D,
        _ => CheckGrade::F,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_findings_block_even_with_passing_score() {
        let score = score_from_factors(&[CheckFactor::critical("slot_lost", 35, "x", "fix")]);

        assert_eq!(score.score, 65);
        assert_eq!(score.grade, CheckGrade::C);
        assert_eq!(score.status, CheckStatus::Blocked);
    }
}
