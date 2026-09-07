#[cfg(test)]
use trellara_protocol::RelationId;

pub const DEFAULT_DRIFT_SAMPLE_LIMIT: usize = 20;

mod canonical_json;
mod comparison;
mod comparison_digest;
mod error;
mod postgres_inspection;
mod postgres_reseed;
mod postgres_reseed_source;
mod postgres_reseed_target;
mod postgres_snapshot;
mod postgres_sql;
mod row_snapshot;
mod table_snapshot_checksum;
mod types;

pub use comparison::{
    compare_snapshots, compare_snapshots_with_sample_limit, ensure_target_caught_up, parse_lsn,
};
pub use error::VerifyError;
pub use postgres_inspection::inspect_postgres_relation;
pub use postgres_reseed::reseed_postgres_table;
#[cfg(test)]
pub(crate) use postgres_reseed_source::source_reseed_select_sql;
pub(crate) use postgres_reseed_source::{copyable_columns, read_source_rows};
pub(crate) use postgres_reseed_target::{ensure_copyable_columns, write_reseed_rows};
pub use postgres_snapshot::snapshot_postgres_table;
pub(crate) use postgres_sql::{
    normalized_row_filter, quote_ident, quote_literal, quote_table, reseed_clear_sql,
    reseed_insert_sql, snapshot_select_sql, CopyableColumn,
};
pub use row_snapshot::RowSnapshot;
pub use types::*;
pub type Result<T> = std::result::Result<T, VerifyError>;

#[cfg(test)]
mod tests;
