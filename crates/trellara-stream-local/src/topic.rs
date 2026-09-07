use crate::{LocalStreamError, Result};

pub(crate) fn validate_topic(topic: &str) -> Result<()> {
    if is_valid_topic(topic) {
        Ok(())
    } else {
        Err(LocalStreamError::InvalidTopic {
            topic: topic.to_string(),
        })
    }
}

fn is_valid_topic(topic: &str) -> bool {
    !topic.is_empty()
        && topic
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_topic_names_safe_for_local_paths() {
        for topic in [
            "trellara.source.dataset.strict",
            "tenant-01",
            "dataset_2026",
            "Topic42",
        ] {
            validate_topic(topic).expect(topic);
        }
    }

    #[test]
    fn rejects_empty_or_path_like_topic_names() {
        for topic in ["", "../escape", "has/slash", "space name", "unicode-µ"] {
            assert!(
                matches!(
                    validate_topic(topic),
                    Err(LocalStreamError::InvalidTopic { .. })
                ),
                "{topic:?} should be rejected"
            );
        }
    }
}
