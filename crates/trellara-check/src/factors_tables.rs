use trellara_pg_capture::TablePreflight;

use crate::CheckFactor;

pub(crate) fn table_factors(tables: &[TablePreflight]) -> Vec<CheckFactor> {
    let mut factors = Vec::new();
    let unsafe_tables = tables
        .iter()
        .filter(|table| !table.exists || !table.update_delete_safe || !table.issues.is_empty())
        .collect::<Vec<_>>();
    if !unsafe_tables.is_empty() {
        factors.push(CheckFactor::critical(
            "table_cdc_unsafe",
            25,
            format!(
                "{} source table(s) are unsafe for UPDATE/DELETE CDC: {}",
                unsafe_tables.len(),
                table_issue_sample(&unsafe_tables)
            ),
            "repair replica identity, primary keys, or table compatibility before enabling CDC",
        ));
    }

    let noted_tables = tables
        .iter()
        .filter(|table| {
            table.exists && table.update_delete_safe && !table.contract_notes.is_empty()
        })
        .collect::<Vec<_>>();
    if !noted_tables.is_empty() {
        factors.push(CheckFactor::warning(
            "table_contract_notes",
            5,
            format!(
                "{} source table(s) have CDC contract notes: {}",
                noted_tables.len(),
                table_note_sample(&noted_tables)
            ),
            "review table contract notes and pin schema fingerprints before production rollout",
        ));
    }

    factors
}

fn table_issue_sample(tables: &[&TablePreflight]) -> String {
    tables
        .iter()
        .take(5)
        .map(|table| {
            let issues = if table.issues.is_empty() {
                "not update/delete safe".to_string()
            } else {
                table.issues.join("; ")
            };
            format!("{} ({issues})", table.qualified_name())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn table_note_sample(tables: &[&TablePreflight]) -> String {
    tables
        .iter()
        .take(5)
        .map(|table| {
            format!(
                "{} ({})",
                table.qualified_name(),
                table.contract_notes.join("; ")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}
