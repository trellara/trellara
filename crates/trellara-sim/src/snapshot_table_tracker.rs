use std::collections::HashSet;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SnapshotTableTracker {
    copied_tables: HashSet<String>,
}

impl SnapshotTableTracker {
    pub(crate) fn mark_copied(&mut self, table: String) {
        self.copied_tables.insert(table);
    }

    pub(crate) fn copied_count(&self) -> usize {
        self.copied_tables.len()
    }

    pub(crate) fn is_complete(&self, table_count: usize) -> bool {
        self.copied_tables.len() == table_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_unique_copied_tables() {
        let mut tracker = SnapshotTableTracker::default();

        tracker.mark_copied("public.sales".to_string());
        tracker.mark_copied("public.sales".to_string());
        tracker.mark_copied("public.customers".to_string());

        assert_eq!(tracker.copied_count(), 2);
    }

    #[test]
    fn reports_completion_against_expected_table_count() {
        let mut tracker = SnapshotTableTracker::default();

        tracker.mark_copied("public.sales".to_string());
        assert!(!tracker.is_complete(2));

        tracker.mark_copied("public.customers".to_string());
        assert!(tracker.is_complete(2));
    }
}
