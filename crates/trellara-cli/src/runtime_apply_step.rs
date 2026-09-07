use crate::{checked_runtime_add, ApplySummary, CliError, Result};
use trellara_protocol::parse_lsn;

impl ApplySummary {
    pub(crate) fn record_step(&mut self, step: trellara_apply_postgres::ApplyStep) -> Result<()> {
        match step.decision {
            trellara_apply_postgres::ApplyDecision::Applied => {
                self.applied_transactions =
                    checked_runtime_add(self.applied_transactions, 1, "applied_transactions")?;
            }
            trellara_apply_postgres::ApplyDecision::SkippedDuplicate => {
                self.skipped_duplicates =
                    checked_runtime_add(self.skipped_duplicates, 1, "skipped_duplicates")?;
            }
        }
        let applied_changes =
            u64::try_from(step.applied_changes).map_err(|_| CliError::RuntimeStatOverflow {
                field: "applied_changes",
            })?;
        self.applied_changes =
            checked_runtime_add(self.applied_changes, applied_changes, "applied_changes")?;
        let acked_messages =
            u64::try_from(step.acked_messages).map_err(|_| CliError::RuntimeStatOverflow {
                field: "acked_messages",
            })?;
        self.acked_messages =
            checked_runtime_add(self.acked_messages, acked_messages, "acked_messages")?;
        self.last_commit_lsn = max_commit_lsn(self.last_commit_lsn.take(), step.commit_lsn)?;
        Ok(())
    }
}

fn max_commit_lsn(current: Option<String>, incoming: String) -> Result<Option<String>> {
    let incoming_lsn = parse_lsn(&incoming)?;
    match current {
        Some(current) if parse_lsn(&current)? > incoming_lsn => Ok(Some(current)),
        _ => Ok(Some(incoming)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellara_apply_postgres::{ApplyDecision, ApplyStep};

    fn step(decision: ApplyDecision, applied_changes: usize, commit_lsn: &str) -> ApplyStep {
        ApplyStep {
            transaction_id: "tx-1".to_string(),
            commit_lsn: commit_lsn.to_string(),
            decision,
            applied_changes,
            acked_messages: 1,
        }
    }

    #[test]
    fn apply_summary_record_step_accumulates_checked_counts() {
        let mut summary = ApplySummary::default();

        summary
            .record_step(step(ApplyDecision::Applied, 3, "0/16B6C50"))
            .expect("applied step");
        summary
            .record_step(step(ApplyDecision::SkippedDuplicate, 0, "0/16B6C50"))
            .expect("duplicate step");

        assert_eq!(summary.applied_transactions, 1);
        assert_eq!(summary.skipped_duplicates, 1);
        assert_eq!(summary.applied_changes, 3);
        assert_eq!(summary.acked_messages, 2);
        assert_eq!(summary.last_commit_lsn, Some("0/16B6C50".to_string()));
    }

    #[test]
    fn apply_summary_record_step_rejects_counter_overflow() {
        let mut summary = ApplySummary {
            applied_changes: u64::MAX,
            ..ApplySummary::default()
        };

        let error = summary
            .record_step(step(ApplyDecision::Applied, 1, "0/16B6C50"))
            .expect_err("overflow");

        assert!(matches!(
            error,
            CliError::RuntimeStatOverflow {
                field: "applied_changes"
            }
        ));
    }

    #[test]
    fn apply_summary_record_step_rejects_acked_message_overflow() {
        let mut summary = ApplySummary {
            acked_messages: u64::MAX,
            ..ApplySummary::default()
        };

        let error = summary
            .record_step(step(ApplyDecision::Applied, 1, "0/16B6C50"))
            .expect_err("overflow");

        assert!(matches!(
            error,
            CliError::RuntimeStatOverflow {
                field: "acked_messages"
            }
        ));
    }

    #[test]
    fn apply_summary_keeps_last_commit_lsn_monotonic_across_replays() {
        let mut summary = ApplySummary::default();

        summary
            .record_step(step(ApplyDecision::Applied, 1, "0/16B7000"))
            .expect("newer step");
        summary
            .record_step(step(ApplyDecision::SkippedDuplicate, 0, "0/16B6C50"))
            .expect("older replay");

        assert_eq!(summary.last_commit_lsn, Some("0/16B7000".to_string()));
        assert_eq!(summary.skipped_duplicates, 1);
    }
}
