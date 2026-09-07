use serde::Serialize;

use crate::{shell_quote, SourceSafetyArgs};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SourceSafetyInitRecommendation {
    pub(crate) command: String,
    pub(crate) output: String,
    pub(crate) table_count: usize,
    pub(crate) primary_key: String,
    pub(crate) evaluation_ready: bool,
    pub(crate) note: String,
}

pub(crate) fn source_safety_init_recommendation(
    args: &SourceSafetyArgs,
    tables: &[trellara_pg_capture::TablePreflight],
) -> Option<SourceSafetyInitRecommendation> {
    let database_url = args.database_url.as_ref()?;
    let existing_tables = tables
        .iter()
        .filter(|table| table.exists)
        .map(trellara_pg_capture::TablePreflight::qualified_name)
        .collect::<Vec<_>>();
    let recommended_tables = if existing_tables.is_empty() {
        args.table.clone()
    } else {
        existing_tables
    };
    if recommended_tables.is_empty() {
        return None;
    }
    let primary_key = tables
        .iter()
        .find_map(|table| table.primary_key_columns.first())
        .cloned()
        .unwrap_or_else(|| "id".to_string());
    let output = args
        .write_init
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "trellara.yml".to_string());
    let target_database_url = args
        .target_database_url
        .as_deref()
        .map(shell_quote)
        .unwrap_or_else(|| "<target-postgres-url>".to_string());

    let mut parts = vec![
        "trellara".to_string(),
        "init".to_string(),
        "--source-database-url".to_string(),
        shell_quote(database_url),
        "--target-database-url".to_string(),
        target_database_url,
        "--source-id".to_string(),
        shell_quote(&args.source_id),
        "--database-id".to_string(),
        shell_quote(&args.database_id),
        "--dataset-id".to_string(),
        shell_quote(&args.dataset_id),
        "--publication".to_string(),
        shell_quote(&args.publication),
        "--slot".to_string(),
        shell_quote(&args.slot),
    ];
    for table in &recommended_tables {
        parts.push("--table".to_string());
        parts.push(shell_quote(table));
    }
    parts.extend([
        "--primary-key".to_string(),
        shell_quote(&primary_key),
        "--output".to_string(),
        shell_quote(&output),
        "--evaluate".to_string(),
    ]);

    Some(SourceSafetyInitRecommendation {
        command: parts.join(" "),
        output,
        table_count: recommended_tables.len(),
        primary_key,
        evaluation_ready: true,
        note: if args.target_database_url.is_some() {
            "run after source-safety findings are acceptable; the generated config is ready for the brokerless verified loop".to_string()
        } else {
            "run after source-safety findings are acceptable; replace <target-postgres-url> with the pilot target database before running the verified loop".to_string()
        },
    })
}
