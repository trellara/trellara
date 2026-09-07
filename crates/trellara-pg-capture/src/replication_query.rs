use crate::lsn::parse_lsn;
use crate::{quote_ident, Result};

pub(crate) fn replication_options_sql(options: &[(&str, &str)]) -> String {
    options
        .iter()
        .map(|(name, value)| format!("{} '{}'", quote_ident(name), value.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn start_logical_replication_query(
    slot_name: &str,
    start_lsn: &str,
    options: &[(&str, &str)],
) -> Result<String> {
    parse_lsn(start_lsn)?;
    Ok(format!(
        "START_REPLICATION SLOT {} LOGICAL {} ({})",
        quote_ident(slot_name),
        start_lsn,
        replication_options_sql(options)
    ))
}
