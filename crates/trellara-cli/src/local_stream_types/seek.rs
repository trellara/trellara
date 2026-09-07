use serde::Serialize;

use super::locate::LocalStreamLocateBoundarySummary;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct LocalStreamSeekSummary {
    pub(crate) root: String,
    pub(crate) group_id: String,
    pub(crate) topic: String,
    pub(crate) previous_next_offset: Option<i64>,
    pub(crate) next_offset: i64,
    pub(crate) topic_message_count: i64,
    pub(crate) pending_before: Option<i64>,
    pub(crate) pending_after: i64,
    pub(crate) movement: String,
    pub(crate) redelivered_messages: i64,
    pub(crate) skipped_messages: i64,
    pub(crate) cursor_status: String,
    pub(crate) allow_ahead: bool,
    pub(crate) configured_topics: Vec<String>,
    pub(crate) boundary: Option<LocalStreamLocateBoundarySummary>,
    pub(crate) boundary_warnings: Vec<String>,
    pub(crate) replay_safe: bool,
    pub(crate) replay_warnings: Vec<String>,
    pub(crate) boundary_seek_commands: Vec<String>,
}
