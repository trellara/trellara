pub(crate) fn local_message_header<'a>(
    message: &'a trellara_stream::StreamMessage,
    key: &str,
) -> Option<&'a str> {
    message
        .headers
        .iter()
        .find(|header| header.key == key)
        .map(|header| header.value.as_str())
}

pub(crate) fn local_message_matches_boundary(
    message: &trellara_stream::StreamMessage,
    transaction_id: &str,
    commit_lsn: &Option<String>,
) -> bool {
    local_message_header(message, "trellara.transaction_id") == Some(transaction_id)
        && match commit_lsn {
            Some(commit_lsn) => {
                local_message_header(message, "trellara.commit_lsn") == Some(commit_lsn.as_str())
            }
            None => true,
        }
}
