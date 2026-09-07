use std::collections::HashSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BaseTarget {
    applied_rows: HashSet<String>,
    dedup: HashSet<String>,
    quarantine: HashSet<String>,
    skipped_duplicates: usize,
}

impl BaseTarget {
    pub(crate) fn has_applied(&self, transaction_id: &str) -> bool {
        self.dedup.contains(transaction_id)
    }

    pub(crate) fn mark_duplicate_skipped(&mut self) {
        self.skipped_duplicates += 1;
    }

    pub(crate) fn quarantine(&mut self, transaction_id: String) {
        self.quarantine.insert(transaction_id);
    }

    pub(crate) fn repair_quarantine(&mut self, transaction_id: &str) {
        self.quarantine.remove(transaction_id);
    }

    pub(crate) fn apply(&mut self, transaction_id: String) {
        self.applied_rows.insert(transaction_id.clone());
        self.dedup.insert(transaction_id);
    }

    pub(crate) fn applied_count(&self) -> usize {
        self.applied_rows.len()
    }

    pub(crate) fn dedup_count(&self) -> usize {
        self.dedup.len()
    }

    pub(crate) fn skipped_duplicate_count(&self) -> usize {
        self.skipped_duplicates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_marks_transaction_for_dedup() {
        let mut target = BaseTarget::default();

        target.apply("tx-1".to_string());

        assert!(target.has_applied("tx-1"));
        assert_eq!(target.applied_count(), 1);
        assert_eq!(target.dedup_count(), 1);
    }

    #[test]
    fn duplicate_skip_counter_is_explicit() {
        let mut target = BaseTarget::default();

        target.mark_duplicate_skipped();
        target.mark_duplicate_skipped();

        assert_eq!(target.skipped_duplicate_count(), 2);
    }

    #[test]
    fn quarantine_repair_removes_transaction_from_quarantine() {
        let mut target = BaseTarget::default();

        target.quarantine("tx-1".to_string());
        target.repair_quarantine("tx-1");

        assert!(target.quarantine.is_empty());
    }
}
