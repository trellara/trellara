use trellara_protocol::parse_lsn;

use crate::{
    checked_worker_stat_add, ApplyDecision, ApplyStep, ApplyWorkerError, ApplyWorkerResult,
    BarrierPendingStats,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ApplyRunStats {
    pub applied_transactions: u64,
    pub skipped_duplicates: u64,
    pub applied_changes: u64,
    pub acked_messages: u64,
    pub last_commit_lsn: Option<String>,
    pub barrier_pending: BarrierPendingStats,
}

impl ApplyRunStats {
    pub(crate) fn record_step(&mut self, step: ApplyStep) -> ApplyWorkerResult<()> {
        if step.decision == ApplyDecision::Applied {
            self.applied_transactions =
                checked_worker_stat_add(self.applied_transactions, 1, "applied_transactions")?;
        }
        if step.decision == ApplyDecision::SkippedDuplicate {
            self.skipped_duplicates =
                checked_worker_stat_add(self.skipped_duplicates, 1, "skipped_duplicates")?;
        }
        let applied_changes =
            u64::try_from(step.applied_changes).map_err(|_| ApplyWorkerError::StatOverflow {
                field: "applied_changes",
            })?;
        self.applied_changes =
            checked_worker_stat_add(self.applied_changes, applied_changes, "applied_changes")?;
        let acked_messages =
            u64::try_from(step.acked_messages).map_err(|_| ApplyWorkerError::StatOverflow {
                field: "acked_messages",
            })?;
        self.acked_messages =
            checked_worker_stat_add(self.acked_messages, acked_messages, "acked_messages")?;
        self.last_commit_lsn = max_commit_lsn(self.last_commit_lsn.take(), step.commit_lsn)?;
        Ok(())
    }
}

fn max_commit_lsn(current: Option<String>, incoming: String) -> ApplyWorkerResult<Option<String>> {
    let incoming_lsn = parse_lsn(&incoming)?;
    match current {
        Some(current) if parse_lsn(&current)? > incoming_lsn => Ok(Some(current)),
        _ => Ok(Some(incoming)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn apply_run_stats_record_step_accumulates_counts() {
        let mut stats = ApplyRunStats::default();

        stats
            .record_step(step(ApplyDecision::Applied, 2, "0/16B6C50"))
            .expect("applied");
        stats
            .record_step(step(ApplyDecision::SkippedDuplicate, 0, "0/16B6C50"))
            .expect("duplicate");

        assert_eq!(stats.applied_transactions, 1);
        assert_eq!(stats.skipped_duplicates, 1);
        assert_eq!(stats.applied_changes, 2);
        assert_eq!(stats.acked_messages, 2);
        assert_eq!(stats.last_commit_lsn, Some("0/16B6C50".to_string()));
    }

    #[test]
    fn apply_run_stats_record_step_rejects_counter_overflow() {
        let mut stats = ApplyRunStats {
            applied_changes: u64::MAX,
            ..ApplyRunStats::default()
        };

        let error = stats
            .record_step(step(ApplyDecision::Applied, 1, "0/16B6C50"))
            .expect_err("overflow");

        assert!(matches!(
            error,
            ApplyWorkerError::StatOverflow {
                field: "applied_changes"
            }
        ));
    }

    #[test]
    fn apply_run_stats_record_step_rejects_acked_message_overflow() {
        let mut stats = ApplyRunStats {
            acked_messages: u64::MAX,
            ..ApplyRunStats::default()
        };

        let error = stats
            .record_step(step(ApplyDecision::Applied, 1, "0/16B6C50"))
            .expect_err("overflow");

        assert!(matches!(
            error,
            ApplyWorkerError::StatOverflow {
                field: "acked_messages"
            }
        ));
    }

    #[test]
    fn apply_run_stats_keeps_last_commit_lsn_monotonic_across_replays() {
        let mut stats = ApplyRunStats::default();

        stats
            .record_step(step(ApplyDecision::Applied, 1, "0/16B7000"))
            .expect("newer transaction");
        stats
            .record_step(step(ApplyDecision::SkippedDuplicate, 0, "0/16B6C50"))
            .expect("older duplicate replay");

        assert_eq!(stats.last_commit_lsn, Some("0/16B7000".to_string()));
        assert_eq!(stats.skipped_duplicates, 1);
    }
}
